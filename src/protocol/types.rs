//! GB26875 协议类型定义

/// GB26875 协议中用到的所有类型定义

/// 系统类型定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SystemType {
    /// 通用
    General = 0,
    /// 火灾报警系统
    FireAlarm = 1,
    /// 预留 (2-9)
    Reserved(u8),
    /// 消防联动控制器
    FireLinkageController = 10,
    /// 消火栓系统
    Hydrant = 11,
    /// 自动喷水灭火系统
    AutoSprinkler = 12,
    /// 气体灭火系统
    GasExtinguishing = 13,
    /// 水喷雾灭火系统（泵启动方式）
    WaterMistPump = 14,
    /// 水喷雾灭火系统（压力容器启动方式）
    WaterMistPressure = 15,
    /// 泡沫灭火系统
    Foam = 16,
    /// 干粉灭火系统
    DryPowder = 17,
    /// 防烟排烟系统
    SmokeControl = 18,
    /// 防火门及卷帘系统
    FireDoor = 19,
    /// 消防电梯
    FireElevator = 20,
    /// 消防应急广播
    EmergencyBroadcast = 21,
    /// 消防应急照明和疏散指示系统
    EmergencyLighting = 22,
    /// 消防电话
    FirePhone = 24,
    /// 预留 (25-127)
    StandardReserved(u8),
    /// 用户自定义 (128-255)
    UserDefined(u8),
}

impl SystemType {
    /// 从字节值创建系统类型
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::General,
            1 => Self::FireAlarm,
            2..=9 => Self::Reserved(value),
            10 => Self::FireLinkageController,
            11 => Self::Hydrant,
            12 => Self::AutoSprinkler,
            13 => Self::GasExtinguishing,
            14 => Self::WaterMistPump,
            15 => Self::WaterMistPressure,
            16 => Self::Foam,
            17 => Self::DryPowder,
            18 => Self::SmokeControl,
            19 => Self::FireDoor,
            20 => Self::FireElevator,
            21 => Self::EmergencyBroadcast,
            22 => Self::EmergencyLighting,
            23 => Self::FireElevator, // 文档中重复了，这里保持一致
            24 => Self::FirePhone,
            25..=127 => Self::StandardReserved(value),
            128..=255 => Self::UserDefined(value),
        }
    }

    /// 转换为字节值
    pub fn to_u8(self) -> u8 {
        match self {
            Self::General => 0,
            Self::FireAlarm => 1,
            Self::Reserved(v) => v,
            Self::FireLinkageController => 10,
            Self::Hydrant => 11,
            Self::AutoSprinkler => 12,
            Self::GasExtinguishing => 13,
            Self::WaterMistPump => 14,
            Self::WaterMistPressure => 15,
            Self::Foam => 16,
            Self::DryPowder => 17,
            Self::SmokeControl => 18,
            Self::FireDoor => 19,
            Self::FireElevator => 20,
            Self::EmergencyBroadcast => 21,
            Self::EmergencyLighting => 22,
            Self::FirePhone => 24,
            Self::StandardReserved(v) => v,
            Self::UserDefined(v) => v,
        }
    }

    /// 是否为用户自定义类型
    pub fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined(_))
    }

    /// 获取类型描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::General => "通用",
            Self::FireAlarm => "火灾报警系统",
            Self::Reserved(_) => "预留",
            Self::FireLinkageController => "消防联动控制器",
            Self::Hydrant => "消火栓系统",
            Self::AutoSprinkler => "自动喷水灭火系统",
            Self::GasExtinguishing => "气体灭火系统",
            Self::WaterMistPump => "水喷雾灭火系统（泵启动方式）",
            Self::WaterMistPressure => "水喷雾灭火系统（压力容器启动方式）",
            Self::Foam => "泡沫灭火系统",
            Self::DryPowder => "干粉灭火系统",
            Self::SmokeControl => "防烟排烟系统",
            Self::FireDoor => "防火门及卷帘系统",
            Self::FireElevator => "消防电梯",
            Self::EmergencyBroadcast => "消防应急广播",
            Self::EmergencyLighting => "消防应急照明和疏散指示系统",
            Self::FirePhone => "消防电话",
            Self::StandardReserved(_) => "标准预留",
            Self::UserDefined(_) => "用户自定义",
        }
    }
}

/// 部件类型定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ComponentType {
    /// 通用
    General = 0,
    /// 火灾报警控制器
    FireAlarmController = 1,
    /// 预留 (2-9)
    Reserved(u8),
    /// 可燃气体探测器
    CombustibleGasDetector = 10,
    /// 点型可燃气体探测器
    PointCombustibleGasDetector = 11,
    /// 独立式可燃气体探测器
    IndependentCombustibleGasDetector = 12,
    /// 线型可燃气体探测器
    LinearCombustibleGasDetector = 13,
    /// 电气火灾监控报警器
    ElectricalFireAlarm = 16,
    /// 剩余电流式电气火灾监控探测器
    ResidualCurrentElectricalFireDetector = 17,
    /// 测温式电气火灾监控探测器
    TemperatureElectricalFireDetector = 18,
    /// 探测回路
    DetectionLoop = 21,
    /// 火灾显示盘
    FireDisplayPanel = 22,
    /// 手动火灾报警按钮
    ManualFireAlarmButton = 23,
    /// 消防栓按钮
    HydrantButton = 24,
    /// 火灾探测器
    FireDetector = 25,
    /// 感温火灾探测器
    TemperatureFireDetector = 30,
    /// 点型感温火灾探测器
    PointTemperatureFireDetector = 31,
    /// 点型感温火灾探测器（S型）
    PointTemperatureFireDetectorS = 32,
    /// 点型感温火灾探测器（R型）
    PointTemperatureFireDetectorR = 33,
    /// 线型感温火灾探测器
    LinearTemperatureFireDetector = 34,
    /// 线型感温火灾探测器（S型）
    LinearTemperatureFireDetectorS = 35,
    /// 线型感温火灾探测器（R型）
    LinearTemperatureFireDetectorR = 36,
    /// 光纤感温火灾探测器
    FiberOpticTemperatureFireDetector = 37,
    /// 感烟火灾探测器
    SmokeFireDetector = 40,
    /// 点型离子感烟火灾探测器
    PointIonSmokeFireDetector = 41,
    /// 点型光电感烟火灾探测器
    PointPhotoelectricSmokeFireDetector = 42,
    /// 线型光束感烟火灾探测器
    LinearBeamSmokeFireDetector = 43,
    /// 吸气式感烟火灾探测器
    AspiratingSmokeFireDetector = 44,
    /// 复合式火灾探测器
    CompositeFireDetector = 50,
    /// 复合式感烟感温火灾探测器
    CompositeSmokeTemperatureFireDetector = 51,
    /// 复合式感光感温火灾探测器
    CompositeLightTemperatureFireDetector = 52,
    /// 复合式感光感烟火灾探测器
    CompositeLightSmokeFireDetector = 53,
    /// 紫外火灾探测器
    UltravioletFireDetector = 61,
    /// 红外火焰探测器
    InfraredFlameDetector = 62,
    /// 感光火灾探测器
    LightFireDetector = 69,
    /// 气体探测器
    GasDetector = 74,
    /// 图像摄像方式火灾探测器
    ImageFireDetector = 78,
    /// 感声火灾探测器
    SoundFireDetector = 79,
    /// 气体灭火控制器
    GasExtinguishingController = 81,
    /// 消防电气控制装置
    FireElectricalController = 82,
    /// 消防控制室图形显示装置
    FireControlRoomDisplay = 83,
    /// 模块
    Module = 84,
    /// 输入模块
    InputModule = 85,
    /// 输出模块
    OutputModule = 86,
    /// 输入/输出模块
    InputOutputModule = 87,
    /// 中继模块
    RelayModule = 88,
    /// 消防水泵
    FirePump = 91,
    /// 消防水箱
    FireWaterTank = 92,
    /// 喷淋泵
    SprinklerPump = 95,
    /// 水流指示器
    WaterFlowIndicator = 96,
    /// 信号阀
    SignalValve = 97,
    /// 报警阀
    AlarmValve = 98,
    /// 压力开关
    PressureSwitch = 99,
    /// 阀驱动装置
    ValveActuator = 101,
    /// 防火门
    FireDoorDevice = 102,
    /// 防火阀
    FireDamper = 103,
    /// 通风空调
    VentilationAirConditioning = 104,
    /// 泡沫液泵
    FoamPump = 105,
    /// 管网电磁阀
    PipelineElectromagneticValve = 106,
    /// 防烟排烟风机
    SmokeExhaustFan = 111,
    /// 排烟防火阀
    SmokeExhaustFireDamper = 113,
    /// 常闭送风口
    NormallyClosedAirInlet = 114,
    /// 排烟口
    SmokeExhaustOutlet = 115,
    /// 电控挡烟垂壁
    ElectricSmokeBarrier = 116,
    /// 防火卷帘控制器
    FireShutterController = 117,
    /// 防火门监控器
    FireDoorMonitor = 118,
    /// 报警装置
    AlarmDevice = 121,
    /// 标准预留 (122-127)
    StandardReserved(u8),
    /// 用户自定义 (128-255)
    UserDefined(u8),
}

impl ComponentType {
    /// 从字节值创建部件类型
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::General,
            1 => Self::FireAlarmController,
            2..=9 => Self::Reserved(value),
            10 => Self::CombustibleGasDetector,
            11 => Self::PointCombustibleGasDetector,
            12 => Self::IndependentCombustibleGasDetector,
            13 => Self::LinearCombustibleGasDetector,
            16 => Self::ElectricalFireAlarm,
            17 => Self::ResidualCurrentElectricalFireDetector,
            18 => Self::TemperatureElectricalFireDetector,
            21 => Self::DetectionLoop,
            22 => Self::FireDisplayPanel,
            23 => Self::ManualFireAlarmButton,
            24 => Self::HydrantButton,
            25 => Self::FireDetector,
            30 => Self::TemperatureFireDetector,
            31 => Self::PointTemperatureFireDetector,
            32 => Self::PointTemperatureFireDetectorS,
            33 => Self::PointTemperatureFireDetectorR,
            34 => Self::LinearTemperatureFireDetector,
            35 => Self::LinearTemperatureFireDetectorS,
            36 => Self::LinearTemperatureFireDetectorR,
            37 => Self::FiberOpticTemperatureFireDetector,
            40 => Self::SmokeFireDetector,
            41 => Self::PointIonSmokeFireDetector,
            42 => Self::PointPhotoelectricSmokeFireDetector,
            43 => Self::LinearBeamSmokeFireDetector,
            44 => Self::AspiratingSmokeFireDetector,
            50 => Self::CompositeFireDetector,
            51 => Self::CompositeSmokeTemperatureFireDetector,
            52 => Self::CompositeLightTemperatureFireDetector,
            53 => Self::CompositeLightSmokeFireDetector,
            61 => Self::UltravioletFireDetector,
            62 => Self::InfraredFlameDetector,
            69 => Self::LightFireDetector,
            74 => Self::GasDetector,
            78 => Self::ImageFireDetector,
            79 => Self::SoundFireDetector,
            81 => Self::GasExtinguishingController,
            82 => Self::FireElectricalController,
            83 => Self::FireControlRoomDisplay,
            84 => Self::Module,
            85 => Self::InputModule,
            86 => Self::OutputModule,
            87 => Self::InputOutputModule,
            88 => Self::RelayModule,
            91 => Self::FirePump,
            92 => Self::FireWaterTank,
            95 => Self::SprinklerPump,
            96 => Self::WaterFlowIndicator,
            97 => Self::SignalValve,
            98 => Self::AlarmValve,
            99 => Self::PressureSwitch,
            101 => Self::ValveActuator,
            102 => Self::FireDoorDevice,
            103 => Self::FireDamper,
            104 => Self::VentilationAirConditioning,
            105 => Self::FoamPump,
            106 => Self::PipelineElectromagneticValve,
            111 => Self::SmokeExhaustFan,
            113 => Self::SmokeExhaustFireDamper,
            114 => Self::NormallyClosedAirInlet,
            115 => Self::SmokeExhaustOutlet,
            116 => Self::ElectricSmokeBarrier,
            117 => Self::FireShutterController,
            118 => Self::FireDoorMonitor,
            121 => Self::AlarmDevice,
            122..=127 => Self::StandardReserved(value),
            128..=255 => Self::UserDefined(value),
            _ => Self::Reserved(value),
        }
    }

    /// 转换为字节值
    pub fn to_u8(self) -> u8 {
        match self {
            Self::General => 0,
            Self::FireAlarmController => 1,
            Self::Reserved(v) => v,
            Self::CombustibleGasDetector => 10,
            Self::PointCombustibleGasDetector => 11,
            Self::IndependentCombustibleGasDetector => 12,
            Self::LinearCombustibleGasDetector => 13,
            Self::ElectricalFireAlarm => 16,
            Self::ResidualCurrentElectricalFireDetector => 17,
            Self::TemperatureElectricalFireDetector => 18,
            Self::DetectionLoop => 21,
            Self::FireDisplayPanel => 22,
            Self::ManualFireAlarmButton => 23,
            Self::HydrantButton => 24,
            Self::FireDetector => 25,
            Self::TemperatureFireDetector => 30,
            Self::PointTemperatureFireDetector => 31,
            Self::PointTemperatureFireDetectorS => 32,
            Self::PointTemperatureFireDetectorR => 33,
            Self::LinearTemperatureFireDetector => 34,
            Self::LinearTemperatureFireDetectorS => 35,
            Self::LinearTemperatureFireDetectorR => 36,
            Self::FiberOpticTemperatureFireDetector => 37,
            Self::SmokeFireDetector => 40,
            Self::PointIonSmokeFireDetector => 41,
            Self::PointPhotoelectricSmokeFireDetector => 42,
            Self::LinearBeamSmokeFireDetector => 43,
            Self::AspiratingSmokeFireDetector => 44,
            Self::CompositeFireDetector => 50,
            Self::CompositeSmokeTemperatureFireDetector => 51,
            Self::CompositeLightTemperatureFireDetector => 52,
            Self::CompositeLightSmokeFireDetector => 53,
            Self::UltravioletFireDetector => 61,
            Self::InfraredFlameDetector => 62,
            Self::LightFireDetector => 69,
            Self::GasDetector => 74,
            Self::ImageFireDetector => 78,
            Self::SoundFireDetector => 79,
            Self::GasExtinguishingController => 81,
            Self::FireElectricalController => 82,
            Self::FireControlRoomDisplay => 83,
            Self::Module => 84,
            Self::InputModule => 85,
            Self::OutputModule => 86,
            Self::InputOutputModule => 87,
            Self::RelayModule => 88,
            Self::FirePump => 91,
            Self::FireWaterTank => 92,
            Self::SprinklerPump => 95,
            Self::WaterFlowIndicator => 96,
            Self::SignalValve => 97,
            Self::AlarmValve => 98,
            Self::PressureSwitch => 99,
            Self::ValveActuator => 101,
            Self::FireDoorDevice => 102,
            Self::FireDamper => 103,
            Self::VentilationAirConditioning => 104,
            Self::FoamPump => 105,
            Self::PipelineElectromagneticValve => 106,
            Self::SmokeExhaustFan => 111,
            Self::SmokeExhaustFireDamper => 113,
            Self::NormallyClosedAirInlet => 114,
            Self::SmokeExhaustOutlet => 115,
            Self::ElectricSmokeBarrier => 116,
            Self::FireShutterController => 117,
            Self::FireDoorMonitor => 118,
            Self::AlarmDevice => 121,
            Self::StandardReserved(v) => v,
            Self::UserDefined(v) => v,
        }
    }

    /// 是否为用户自定义类型
    pub fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined(_))
    }
}

/// 模拟量类型定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AnalogType {
    /// 未用
    Unused = 0,
    /// 事件计数（件）
    EventCount = 1,
    /// 高度（m）
    Height = 2,
    /// 温度（℃）
    Temperature = 3,
    /// 压力（MPa）
    PressureMPa = 4,
    /// 压力（kPa）
    PressureKPa = 5,
    /// 气体浓度（%LEL）
    GasConcentration = 6,
    /// 时间（s）
    Time = 7,
    /// 电压（V）
    Voltage = 8,
    /// 电流（A）
    Current = 9,
    /// 流量（L/s）
    Flow = 10,
    /// 风量（m³/min）
    AirFlow = 11,
    /// 风速（m/s）
    WindSpeed = 12,
    /// 预留 (13-127)
    Reserved(u8),
    /// 用户自定义 (128-255)
    UserDefined(u8),
}

impl AnalogType {
    /// 从字节值创建模拟量类型
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unused,
            1 => Self::EventCount,
            2 => Self::Height,
            3 => Self::Temperature,
            4 => Self::PressureMPa,
            5 => Self::PressureKPa,
            6 => Self::GasConcentration,
            7 => Self::Time,
            8 => Self::Voltage,
            9 => Self::Current,
            10 => Self::Flow,
            11 => Self::AirFlow,
            12 => Self::WindSpeed,
            13..=127 => Self::Reserved(value),
            128..=255 => Self::UserDefined(value),
        }
    }

    /// 转换为字节值
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Unused => 0,
            Self::EventCount => 1,
            Self::Height => 2,
            Self::Temperature => 3,
            Self::PressureMPa => 4,
            Self::PressureKPa => 5,
            Self::GasConcentration => 6,
            Self::Time => 7,
            Self::Voltage => 8,
            Self::Current => 9,
            Self::Flow => 10,
            Self::AirFlow => 11,
            Self::WindSpeed => 12,
            Self::Reserved(v) => v,
            Self::UserDefined(v) => v,
        }
    }

    /// 是否为用户自定义类型
    pub fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined(_))
    }

    /// 获取单位字符串
    pub fn unit(&self) -> &'static str {
        match self {
            Self::Unused => "",
            Self::EventCount => "件",
            Self::Height => "m",
            Self::Temperature => "℃",
            Self::PressureMPa => "MPa",
            Self::PressureKPa => "kPa",
            Self::GasConcentration => "%LEL",
            Self::Time => "s",
            Self::Voltage => "V",
            Self::Current => "A",
            Self::Flow => "L/s",
            Self::AirFlow => "m³/min",
            Self::WindSpeed => "m/s",
            Self::Reserved(_) => "",
            Self::UserDefined(_) => "",
        }
    }

    /// 获取最小计量单元
    pub fn min_unit(&self) -> f32 {
        match self {
            Self::Unused => 0.0,
            Self::EventCount => 1.0,
            Self::Height => 0.01,
            Self::Temperature => 0.1,
            Self::PressureMPa => 0.1,
            Self::PressureKPa => 0.1,
            Self::GasConcentration => 0.1,
            Self::Time => 1.0,
            Self::Voltage => 0.1,
            Self::Current => 0.1,
            Self::Flow => 0.1,
            Self::AirFlow => 0.1,
            Self::WindSpeed => 1.0,
            Self::Reserved(_) => 0.0,
            Self::UserDefined(_) => 0.0,
        }
    }

    /// 获取描述字符串
    pub fn description(&self) -> &'static str {
        match self {
            Self::Unused => "未用",
            Self::EventCount => "事件计数",
            Self::Height => "高度",
            Self::Temperature => "温度",
            Self::PressureMPa => "压力（MPa）",
            Self::PressureKPa => "压力（kPa）",
            Self::GasConcentration => "气体浓度",
            Self::Time => "时间",
            Self::Voltage => "电压",
            Self::Current => "电流",
            Self::Flow => "流量",
            Self::AirFlow => "风量",
            Self::WindSpeed => "风速",
            Self::Reserved(_) => "预留",
            Self::UserDefined(_) => "用户自定义",
        }
    }

    /// 有效值范围
    pub fn valid_range(&self) -> Option<(i16, i16)> {
        match self {
            Self::Unused => None,
            Self::EventCount => Some((0, 32000)),
            Self::Height => Some((0, 320)),             // 0.00m - 3.20m
            Self::Temperature => Some((-273, 3200)),    // -27.3℃ - 320℃
            Self::PressureMPa => Some((0, 3200)),       // 0.0MPa - 100.0MPa
            Self::PressureKPa => Some((0, 3200)),       // 0.0kPa - 1000.0kPa
            Self::GasConcentration => Some((0, 100)),   // 0.0%LEL - 100.0%LEL
            Self::Time => Some((0, 32000)),
            Self::Voltage => Some((0, 3200)), // 0.0V - 320.0V
            Self::Current => Some((0, 3200)), // 0.0A - 320.0A
            Self::Flow => Some((0, 32767)), // 修正为i16最大值
            Self::AirFlow => Some((0, 3200)), // 0.0m³/min - 320.0m³/min
            Self::WindSpeed => Some((0, 20)), // 0m/s - 20m/s
            Self::Reserved(_) => None,
            Self::UserDefined(_) => None,
        }
    }
}

/// 数据单元类型标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum DataUnitType {
    /// 预留
    Reserved = 0,
    /// 上传建筑消防设施系统状态
    UploadSystemStatus = 1,
    /// 上传建筑消防设施部件运行状态
    UploadComponentStatus = 2,
    /// 上传建筑消防设施部件模拟量值
    UploadAnalogValue = 3,
    /// 上传建筑消防设施操作信息
    UploadOperationInfo = 4,
    /// 上传建筑消防设施软件版本
    UploadSoftwareVersion = 5,
    /// 上传建筑消防设施系统配置情况
    UploadSystemConfig = 6,
    /// 上传建筑消防设施部件配置情况
    UploadComponentConfig = 7,
    /// 上传建筑消防设施系统时间
    UploadSystemTime = 8,
    /// 预留（建筑消防设施信息）(9-20)
    FireSystemReserved(u8),
    /// 上传用户信息传输装置运行状态
    UploadDeviceStatus = 21,
    /// 预留 (22-23)
    DeviceReserved1(u8),
    /// 上传用户信息传输装置操作信息
    UploadDeviceOperation = 24,
    /// 上传用户信息传输装置软件版本
    UploadDeviceVersion = 25,
    /// 上传用户信息传输装置配置情况
    UploadDeviceConfig = 26,
    /// 预留 (27)
    DeviceReserved2(u8),
    /// 上传用户信息传输装置系统时间
    UploadDeviceTime = 28,
    /// 预留（用户信息传输装置信息）(29-40)
    DeviceReserved3(u8),
    /// 预留（控制信息）(41-60)
    ControlReserved(u8),
    /// 读建筑消防设施系统状态
    ReadSystemStatus = 61,
    /// 读建筑消防设施部件运行状态
    ReadComponentStatus = 62,
    /// 读建筑消防设施模拟量值
    ReadAnalogValue = 63,
    /// 读建筑消防设施操作信息
    ReadOperationInfo = 64,
    /// 读建筑消防设施软件版本
    ReadSoftwareVersion = 65,
    /// 读建筑消防设施系统配置情况
    ReadSystemConfig = 66,
    /// 读建筑消防设施部件配置情况
    ReadComponentConfig = 67,
    /// 读建筑消防设施系统时间
    ReadSystemTime = 68,
    /// 预留 (69-80)
    ReadFireSystemReserved(u8),
    /// 读用户信息传输装置运行状态
    ReadDeviceStatus = 81,
    /// 预留 (82-83)
    ReadDeviceReserved1(u8),
    /// 读用户信息传输装置操作信息记录
    ReadDeviceOperation = 84,
    /// 读用户信息传输装置软件版本
    ReadDeviceVersion = 85,
    /// 读用户信息传输装置配置情况
    ReadDeviceConfig = 86,
    /// 预留 (87)
    ReadDeviceReserved2(u8),
    /// 读用户信息传输装置系统时间
    ReadDeviceTime = 88,
    /// 初始化用户信息传输装置
    InitializeDevice = 89,
    /// 同步用户信息传输装置时钟
    SyncDeviceClock = 90,
    /// 查岗命令
    PatrolCommand = 91,
    /// 预留 (92-127)
    StandardReserved(u8),
    /// 用户自定义 (128-254)
    UserDefined(u8),
}

impl DataUnitType {
    /// 从字节值创建数据单元类型
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Reserved,
            1 => Self::UploadSystemStatus,
            2 => Self::UploadComponentStatus,
            3 => Self::UploadAnalogValue,
            4 => Self::UploadOperationInfo,
            5 => Self::UploadSoftwareVersion,
            6 => Self::UploadSystemConfig,
            7 => Self::UploadComponentConfig,
            8 => Self::UploadSystemTime,
            9..=20 => Self::FireSystemReserved(value),
            21 => Self::UploadDeviceStatus,
            22..=23 => Self::DeviceReserved1(value),
            24 => Self::UploadDeviceOperation,
            25 => Self::UploadDeviceVersion,
            26 => Self::UploadDeviceConfig,
            27 => Self::DeviceReserved2(value),
            28 => Self::UploadDeviceTime,
            29..=40 => Self::DeviceReserved3(value),
            41..=60 => Self::ControlReserved(value),
            61 => Self::ReadSystemStatus,
            62 => Self::ReadComponentStatus,
            63 => Self::ReadAnalogValue,
            64 => Self::ReadOperationInfo,
            65 => Self::ReadSoftwareVersion,
            66 => Self::ReadSystemConfig,
            67 => Self::ReadComponentConfig,
            68 => Self::ReadSystemTime,
            69..=80 => Self::ReadFireSystemReserved(value),
            81 => Self::ReadDeviceStatus,
            82..=83 => Self::ReadDeviceReserved1(value),
            84 => Self::ReadDeviceOperation,
            85 => Self::ReadDeviceVersion,
            86 => Self::ReadDeviceConfig,
            87 => Self::ReadDeviceReserved2(value),
            88 => Self::ReadDeviceTime,
            89 => Self::InitializeDevice,
            90 => Self::SyncDeviceClock,
            91 => Self::PatrolCommand,
            92..=127 => Self::StandardReserved(value),
            128..=254 => Self::UserDefined(value),
            255 => Self::StandardReserved(value), // 255 不在用户自定义范围内
        }
    }

    /// 转换为字节值
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Reserved => 0,
            Self::UploadSystemStatus => 1,
            Self::UploadComponentStatus => 2,
            Self::UploadAnalogValue => 3,
            Self::UploadOperationInfo => 4,
            Self::UploadSoftwareVersion => 5,
            Self::UploadSystemConfig => 6,
            Self::UploadComponentConfig => 7,
            Self::UploadSystemTime => 8,
            Self::FireSystemReserved(v) => v,
            Self::UploadDeviceStatus => 21,
            Self::DeviceReserved1(v) => v,
            Self::UploadDeviceOperation => 24,
            Self::UploadDeviceVersion => 25,
            Self::UploadDeviceConfig => 26,
            Self::DeviceReserved2(v) => v,
            Self::UploadDeviceTime => 28,
            Self::DeviceReserved3(v) => v,
            Self::ControlReserved(v) => v,
            Self::ReadSystemStatus => 61,
            Self::ReadComponentStatus => 62,
            Self::ReadAnalogValue => 63,
            Self::ReadOperationInfo => 64,
            Self::ReadSoftwareVersion => 65,
            Self::ReadSystemConfig => 66,
            Self::ReadComponentConfig => 67,
            Self::ReadSystemTime => 68,
            Self::ReadFireSystemReserved(v) => v,
            Self::ReadDeviceStatus => 81,
            Self::ReadDeviceReserved1(v) => v,
            Self::ReadDeviceOperation => 84,
            Self::ReadDeviceVersion => 85,
            Self::ReadDeviceConfig => 86,
            Self::ReadDeviceReserved2(v) => v,
            Self::ReadDeviceTime => 88,
            Self::InitializeDevice => 89,
            Self::SyncDeviceClock => 90,
            Self::PatrolCommand => 91,
            Self::StandardReserved(v) => v,
            Self::UserDefined(v) => v,
        }
    }

    /// 是否为用户自定义类型
    pub fn is_user_defined(&self) -> bool {
        matches!(self, Self::UserDefined(_))
    }

    /// 是否为上行数据类型
    pub fn is_upstream(&self) -> bool {
        matches!(self.to_u8(), 1..=28)
    }    /// 是否为下行数据类型
    pub fn is_downstream(&self) -> bool {
        matches!(self.to_u8(), 61..=91)
    }

    /// 获取描述字符串
    pub fn description(&self) -> &'static str {
        match self {
            Self::Reserved => "预留",
            Self::UploadSystemStatus => "上传建筑消防设施系统状态",
            Self::UploadComponentStatus => "上传建筑消防设施部件运行状态",
            Self::UploadAnalogValue => "上传建筑消防设施部件模拟量值",
            Self::UploadOperationInfo => "上传建筑消防设施操作信息",
            Self::UploadSoftwareVersion => "上传建筑消防设施软件版本",
            Self::UploadSystemConfig => "上传建筑消防设施系统配置情况",
            Self::UploadComponentConfig => "上传建筑消防设施部件配置情况",
            Self::UploadSystemTime => "上传建筑消防设施系统时间",
            Self::FireSystemReserved(_) => "预留（建筑消防设施信息）",
            Self::UploadDeviceStatus => "上传用户信息传输装置运行状态",
            Self::DeviceReserved1(_) => "预留",
            Self::UploadDeviceOperation => "上传用户信息传输装置操作信息",
            Self::UploadDeviceVersion => "上传用户信息传输装置软件版本",
            Self::UploadDeviceConfig => "上传用户信息传输装置配置情况",
            Self::DeviceReserved2(_) => "预留",
            Self::UploadDeviceTime => "上传用户信息传输装置系统时间",
            Self::DeviceReserved3(_) => "预留（用户信息传输装置信息）",
            Self::ControlReserved(_) => "预留（控制信息）",
            Self::ReadSystemStatus => "读建筑消防设施系统状态",
            Self::ReadComponentStatus => "读建筑消防设施部件运行状态",
            Self::ReadAnalogValue => "读建筑消防设施模拟量值",
            Self::ReadOperationInfo => "读建筑消防设施操作信息",
            Self::ReadSoftwareVersion => "读建筑消防设施软件版本",
            Self::ReadSystemConfig => "读建筑消防设施系统配置情况",
            Self::ReadComponentConfig => "读建筑消防设施部件配置情况",
            Self::ReadSystemTime => "读建筑消防设施系统时间",
            Self::ReadFireSystemReserved(_) => "预留",
            Self::ReadDeviceStatus => "读用户信息传输装置运行状态",
            Self::ReadDeviceReserved1(_) => "预留",
            Self::ReadDeviceOperation => "读用户信息传输装置操作信息",
            Self::ReadDeviceVersion => "读用户信息传输装置软件版本",
            Self::ReadDeviceConfig => "读用户信息传输装置配置情况",
            Self::ReadDeviceReserved2(_) => "预留",
            Self::ReadDeviceTime => "读用户信息传输装置系统时间",
            Self::InitializeDevice => "初始化用户信息传输装置",
            Self::SyncDeviceClock => "同步用户信息传输装置时钟",
            Self::PatrolCommand => "查岗命令",
            Self::StandardReserved(_) => "预留",
            Self::UserDefined(_) => "用户自定义",
        }
    }
}