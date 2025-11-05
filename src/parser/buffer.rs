//! GB26875 缓冲区管理模块
//!
//! 提供缓冲区管理功能，用于处理TCP粘包和数据流缓冲

use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;

/// 流缓冲区管理器
///
/// 用于管理来自网络流的数据缓冲，处理粘包和分包情况
#[derive(Debug, Clone)]
pub struct StreamBuffer {
    /// 主缓冲区
    buffer: BytesMut,
    /// 待处理的完整帧队列
    frame_queue: VecDeque<Bytes>,
    /// 最大缓冲区大小
    max_buffer_size: usize,
    /// 统计信息
    stats: BufferStats,
}

/// 缓冲区统计信息
#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    /// 总接收字节数
    pub bytes_received: u64,
    /// 总处理的帧数
    pub frames_processed: u64,
    /// 缓冲区溢出次数
    pub buffer_overflows: u64,
    /// 当前缓冲区大小
    pub current_buffer_size: usize,
    /// 峰值缓冲区大小
    pub peak_buffer_size: usize,
}

impl StreamBuffer {
    /// 创建新的流缓冲区
    pub fn new() -> Self {
        Self::with_capacity(8192) // 默认8KB容量
    }

    /// 创建带指定容量的流缓冲区
    ///
    /// # Arguments
    /// * `capacity` - 初始容量
    pub fn with_capacity(capacity: usize) -> Self {
        StreamBuffer {
            buffer: BytesMut::with_capacity(capacity),
            frame_queue: VecDeque::new(),
            max_buffer_size: capacity * 4, // 最大为初始容量的4倍
            stats: BufferStats::default(),
        }
    }

    /// 设置最大缓冲区大小
    ///
    /// # Arguments
    /// * `max_size` - 最大缓冲区大小（字节）
    pub fn set_max_buffer_size(&mut self, max_size: usize) {
        self.max_buffer_size = max_size;
    }

    /// 向缓冲区添加数据
    ///
    /// # Arguments
    /// * `data` - 新接收的数据
    ///
    /// # Returns
    /// * `Ok(())` - 成功添加
    /// * `Err(BufferError)` - 缓冲区满或其他错误
    pub fn push_data(&mut self, data: &[u8]) -> Result<(), BufferError> {
        // 检查缓冲区大小限制
        if self.buffer.len() + data.len() > self.max_buffer_size {
            self.stats.buffer_overflows += 1;
            return Err(BufferError::BufferOverflow {
                current_size: self.buffer.len(),
                incoming_size: data.len(),
                max_size: self.max_buffer_size,
            });
        }

        // 添加数据到缓冲区
        self.buffer.extend_from_slice(data);

        // 更新统计信息
        self.stats.bytes_received += data.len() as u64;
        self.stats.current_buffer_size = self.buffer.len();
        if self.buffer.len() > self.stats.peak_buffer_size {
            self.stats.peak_buffer_size = self.buffer.len();
        }

        Ok(())
    }

    /// 从缓冲区中提取指定长度的数据
    ///
    /// # Arguments
    /// * `length` - 要提取的数据长度
    ///
    /// # Returns
    /// * `Some(Bytes)` - 成功提取的数据
    /// * `None` - 缓冲区中的数据不足
    pub fn extract(&mut self, length: usize) -> Option<Bytes> {
        if self.buffer.len() >= length {
            let extracted = self.buffer.split_to(length);
            self.stats.current_buffer_size = self.buffer.len();
            Some(extracted.freeze())
        } else {
            None
        }
    }

    /// 查看缓冲区中的数据而不消耗它
    ///
    /// # Arguments
    /// * `length` - 要查看的数据长度
    ///
    /// # Returns
    /// * `Some(&[u8])` - 可查看的数据切片
    /// * `None` - 缓冲区中的数据不足
    pub fn peek(&self, length: usize) -> Option<&[u8]> {
        if self.buffer.len() >= length {
            Some(&self.buffer[..length])
        } else {
            None
        }
    }

    /// 查看缓冲区中的所有数据
    pub fn peek_all(&self) -> &[u8] {
        &self.buffer
    }

    /// 丢弃缓冲区开头的指定字节数
    ///
    /// # Arguments
    /// * `count` - 要丢弃的字节数
    ///
    /// # Returns
    /// * `usize` - 实际丢弃的字节数
    pub fn discard(&mut self, count: usize) -> usize {
        let actual_count = count.min(self.buffer.len());
        if actual_count > 0 {
            let _ = self.buffer.split_to(actual_count);
            self.stats.current_buffer_size = self.buffer.len();
        }
        actual_count
    }

    /// 压缩缓冲区，回收未使用的容量
    pub fn compact(&mut self) {
        // 如果缓冲区使用率较低，重新分配以释放内存
        if self.buffer.capacity() > 1024 && self.buffer.len() < self.buffer.capacity() / 4 {
            let mut new_buffer = BytesMut::with_capacity(self.buffer.len() * 2);
            new_buffer.extend_from_slice(&self.buffer);
            self.buffer = new_buffer;
        }
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.frame_queue.clear();
        self.stats.current_buffer_size = 0;
    }

    /// 获取当前缓冲区大小
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// 获取剩余容量
    pub fn remaining_capacity(&self) -> usize {
        self.max_buffer_size.saturating_sub(self.buffer.len())
    }

    /// 获取统计信息
    pub fn stats(&self) -> &BufferStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = BufferStats {
            current_buffer_size: self.buffer.len(),
            peak_buffer_size: self.buffer.len(),
            ..Default::default()
        };
    }

    /// 将完整的帧添加到队列中
    ///
    /// # Arguments
    /// * `frame` - 完整的帧数据
    pub fn enqueue_frame(&mut self, frame: Bytes) {
        self.frame_queue.push_back(frame);
        self.stats.frames_processed += 1;
    }

    /// 从队列中获取下一个完整的帧
    ///
    /// # Returns
    /// * `Some(Bytes)` - 下一个完整的帧
    /// * `None` - 队列为空
    pub fn dequeue_frame(&mut self) -> Option<Bytes> {
        self.frame_queue.pop_front()
    }

    /// 获取队列中的帧数量
    pub fn frame_queue_len(&self) -> usize {
        self.frame_queue.len()
    }

    /// 检查帧队列是否为空
    pub fn frame_queue_is_empty(&self) -> bool {
        self.frame_queue.is_empty()
    }
}

impl Default for StreamBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓冲区错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum BufferError {
    /// 缓冲区溢出
    #[error("Buffer overflow: current={current_size}, incoming={incoming_size}, max={max_size}")]
    BufferOverflow {
        /// 当前缓冲区大小
        current_size: usize,
        /// 要添加的数据大小
        incoming_size: usize,
        /// 最大允许大小
        max_size: usize,
    },

    /// 数据不足
    #[error("Insufficient data: required={required}, available={available}")]
    InsufficientData {
        /// 需要的数据量
        required: usize,
        /// 可用的数据量
        available: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_buffer_basic() {
        let mut buffer = StreamBuffer::new();

        // 测试添加数据
        let data = b"Hello, World!";
        buffer.push_data(data).unwrap();
        assert_eq!(buffer.len(), data.len());

        // 测试提取数据
        let extracted = buffer.extract(5).unwrap();
        assert_eq!(extracted.as_ref(), b"Hello");
        assert_eq!(buffer.len(), data.len() - 5);

        // 测试查看数据
        let peeked = buffer.peek(2).unwrap();
        assert_eq!(peeked, b", ");
        assert_eq!(buffer.len(), data.len() - 5); // 查看不应该消耗数据
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buffer = StreamBuffer::with_capacity(10);
        buffer.set_max_buffer_size(20);

        // 添加数据直到接近限制
        buffer.push_data(b"1234567890").unwrap();
        buffer.push_data(b"abcdefghij").unwrap();

        // 尝试添加超出限制的数据
        let result = buffer.push_data(b"overflow");
        assert!(result.is_err());

        match result {
            Err(BufferError::BufferOverflow { .. }) => {
                // 期望的错误类型
            }
            _ => panic!("Expected BufferOverflow error"),
        }
    }

    #[test]
    fn test_frame_queue() {
        let mut buffer = StreamBuffer::new();

        // 添加帧到队列
        let frame1 = Bytes::from_static(b"frame1");
        let frame2 = Bytes::from_static(b"frame2");

        buffer.enqueue_frame(frame1.clone());
        buffer.enqueue_frame(frame2.clone());

        assert_eq!(buffer.frame_queue_len(), 2);
        assert!(!buffer.frame_queue_is_empty());

        // 从队列中取出帧
        let dequeued1 = buffer.dequeue_frame().unwrap();
        assert_eq!(dequeued1, frame1);

        let dequeued2 = buffer.dequeue_frame().unwrap();
        assert_eq!(dequeued2, frame2);

        assert_eq!(buffer.frame_queue_len(), 0);
        assert!(buffer.frame_queue_is_empty());
    }

    #[test]
    fn test_discard_and_compact() {
        let mut buffer = StreamBuffer::new();

        // 添加一些数据
        buffer.push_data(b"1234567890abcdefghij").unwrap();
        assert_eq!(buffer.len(), 20);

        // 丢弃前10个字节
        let discarded = buffer.discard(10);
        assert_eq!(discarded, 10);
        assert_eq!(buffer.len(), 10);

        // 检查剩余数据
        let peeked = buffer.peek_all();
        assert_eq!(peeked, b"abcdefghij");

        // 测试压缩
        buffer.compact();
        // 压缩后数据应该保持不变
        assert_eq!(buffer.peek_all(), b"abcdefghij");
    }

    #[test]
    fn test_stats() {
        let mut buffer = StreamBuffer::new();

        // 添加数据并检查统计
        buffer.push_data(b"test data").unwrap();
        let stats = buffer.stats();
        assert_eq!(stats.bytes_received, 9);
        assert_eq!(stats.current_buffer_size, 9);
        assert!(stats.peak_buffer_size >= 9);

        // 重置统计
        buffer.reset_stats();
        let stats = buffer.stats();
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.frames_processed, 0);
    }
}
