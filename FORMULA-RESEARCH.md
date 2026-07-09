# 选型公式调研记录

本文档用于后续重构计算模块。目标不是复刻某个内部 PDF，而是按设计师实际已知工况，结合公开工程公式、厂家技术资料和内部经验做可复算的选型计算。

## 1. 调研原则

- 用户界面不展示 PDF 页码、内部资料名或“按某 PDF”的痕迹。
- 每个模块先定义设计师通常已知的输入，再定义公式，而不是按资料章节顺序照搬字段。
- 公式必须代码化、可测试、可导出过程。
- 安全系数由用户手动输入并确认。
- 默认值只能作为建议，不能替代用户判断。
- 厂家样本用于型号匹配时，必须显示匹配条件和失败原因。

## 2. 已确认的公开资料起点

| 类别 | 资料来源 | 用途 |
|---|---|---|
| 电机/伺服/步进 | Oriental Motor - Motor Sizing Calculations: https://www.orientalmotor.com/technology/motor-sizing-calculations.html | 负载转矩、加速转矩、惯量、速度、停止精度等计算框架 |
| 电机/伺服/步进 | Oriental Motor - Selection Procedures and Calculation Formulas: https://www.orientalmotor.eu/it/tech/calculation/sizing-motor04 | 所需转矩 = 负载转矩 + 加速转矩，并乘安全系数的计算流程 |
| 滚珠丝杠 | THK - Permissible Rotational Speed: https://www.thk.com/jp/en/products/ball_screw/selection/0007/ | 临界转速、DN 值和允许转速校核 |
| 滚珠丝杠 | THK - Studying the Service Life: https://www.thk.com/jp/en/products/ball_screw/selection/0010/ | 额定寿命、平均转速、载荷系数和寿命校核 |
| 滚珠丝杠 | THK - Example Ball Screw Selections: https://tech.thk.com/en/products/pdf/en_b15_069.pdf | 选型样例、寿命、行程寿命和临界转速计算样式 |
| 气缸 | SMC - Air Cylinders Model Selection: https://www.smcworld.com/catalog/BEST-technical-data-en/pdf/6-2-1-m21-43-tech_en.pdf | 缸径选择、理论输出力、负载率和速度相关选型 |
| 气缸 | SMC - Theoretical Output Table: https://www.smcworld.com/assets/select_guide/en-jp/actuator/pdf/riron-e.pdf | 不同缸径、杆径、压力下的理论输出力核对 |
| 气动控制/电磁阀 | SMC - Air Cylinders Model Selection Data 2: https://www.smcworld.com/catalog/BEST-technical-data-en/pdf/6-2-1-m21-43-tech_en.pdf | 气缸耗气量、管路耗气量、每分钟所需空气量 |
| 同步带/V 带 | Gates - Light Power and Precision Drive Design Manual: https://www.gates.com/content/dam/documents-library/catalogs/light-power-and-precision-manual.pdf | 同步带、精密传动、带轮和功率选型流程 |
| V 带 | Gates - Heavy Duty V-Belt Drive Design Manual: https://www.gates.com/content/dam/documents-library/catalogs/heavy-duty-vbelt-drive-design-manual-en.pdf | V 带功率、工况系数和传动设计校核 |
| 凸轮分割器 | Sankyo ECO Series: https://www.sankyo-seisakusho.co.jp/english/download/pdf/catalog/sandex/c_sandex_eco_ed_eng.pdf | 分割器选型需确认输入/输出条件、负载条件，并由扭矩计算复核 |
| 凸轮分割器 | DESTACO CAMCO Mechanical Rotary Indexers: https://www.destaco.com/rotary-positioning/indexers/mechanical-rotary-indexers | 分割器用于精确间歇定位，型号需结合工位、index time、负载和扭矩能力 |
| 滚动轴承 | SKF - Rating life: https://www.skf.com/group/products/rolling-bearings/principles-of-rolling-bearing-selection/bearing-selection-process/bearing-size/size-selection-based-on-rating-life/skf-rating-life | L10 寿命、等效动载荷和 C/P 关系 |
| 直线轴承 | THK - Rated Load and Nominal Life: https://www.thk.com/eu/en/products/other_linear_motion_guides/linear_bushing/selection/0002/ | 直线轴承 50 km 名义寿命和 C/P 寿命公式 |
| 联轴器 | SKF Couplings Catalogue: https://cdn.skfmediahub.skf.com/api/public/094e20a34cf10d47/pdf_preview_medium/15822_%28EN%29_SKF_Couplings_pdf_preview_medium.pdf | 联轴器额定值 = 工况系数 × 系统扭矩，且需按转速和安装偏差复核 |
| 联轴器 | KTR Coupling Selection Operating Factors: https://www.ktr.com/fileadmin/ktr/media/Tools_Downloads/kataloge/coupling_selection_operating_factors.pdf | 额定扭矩、最大扭矩、温度系数和冲击工况复核 |

## 3. 高频模块输入重构方向

### 2026-07-09 已落地实现

| 模块 | 已调整输入 | 已调整输出 / 风险 |
|---|---|---|
| 伺服/步进 | 外部阻力、垂直负载系数 | 总力矩覆盖摩擦力、加速力、垂直负载力、外部阻力 |
| 滚珠丝杠 | 外部轴向力、垂直负载系数、支撑跨距、底径、支撑方式系数、额定动载荷、目标行走寿命 | 临界转速、估算行走寿命、转速风险、寿命风险 |
| 气缸 | 外部阻力、垂直负载系数、有效面积系数 | 输出力、缸径需求、垂直负载力 |
| 真空吸附 | 姿态修正系数、有效吸附率 | 姿态修正后的吸附力和吸盘直径 |
| 直线导轨 | 目标行走寿命 | 额定寿命估算和寿命风险 |
| 同步带 | 外部阻力、垂直负载系数 | 等效推力覆盖摩擦力、加速力、垂直负载力和外部阻力 |
| 普通电机/调速电机 | 启动加速时间、外部阻力、垂直负载系数 | 输出等效推力、启动加速力、功率、转速和调速范围风险 |
| 电磁阀/气动控制 | 气缸数量、杆径、管路内径、管路长度、阀额定流量 | 单循环耗气、峰值流量、持续耗气量和阀流量余量 |
| 凸轮分割器 | 分割时间、运动曲线系数 | 停歇时间、峰值角速度、角加速度、设计扭矩和峰值功率 |
| 滚动轴承 | X/Y 载荷系数、目标寿命、C/C0、寿命指数 | L10 寿命、所需动额定载荷、动载余量和静载余量 |
| 直线轴承 | 轴承数量、方向系数、冲击系数、目标行走寿命 | 单轴承设计载荷、50 km 基准寿命、所需 C 值和速度风险 |
| 联轴器 | 峰值扭矩、候选额定扭矩、温度系数、角向/轴向偏差 | 设计扭矩、扭矩余量、扭转需求指标和偏差指标 |

### 伺服电机 / 步进电机

设计师通常已知：

- 负载质量或外部负载力
- 移动方式：同步带、丝杠、转台、链条、齿轮等
- 行程或转角
- 目标速度
- 加速时间
- 传动效率
- 摩擦系数
- 每转移动量或减速比
- 安全系数
- 目标定位精度或分辨率

输出：

- 负载转矩
- 加速转矩
- 所需转矩
- 需求转速
- 负载惯量
- 惯量比
- 分辨率校核
- 风险提示

### 气缸

设计师通常已知：

- 负载质量或推/拉负载
- 动作方向：水平、垂直上升、垂直下降、夹紧
- 工作压力
- 行程
- 动作时间或目标速度
- 负载率
- 安全系数

输出：

- 理论输出力
- 选型输出力
- 建议缸径
- 速度/缓冲风险
- 垂直负载风险

### 真空吸附

设计师通常已知：

- 工件质量
- 吸附方向：水平吸附、垂直吊取、侧向移动
- 真空度
- 吸盘数量
- 加速度或搬运节拍
- 安全系数
- 工件表面情况

输出：

- 单个吸盘所需吸附力
- 所需吸附面积
- 估算吸盘直径
- 真空度不足风险
- 多吸盘分布风险

### 同步轮同步带

设计师通常已知：

- 负载质量
- 目标线速度
- 加速时间
- 带轮齿数
- 齿距
- 传动效率
- 摩擦系数
- 外部阻力
- 垂直负载系数
- 安全系数

输出：

- 摩擦力
- 加速力
- 垂直负载力
- 等效推力
- 输出转矩
- 需求转速
- 估算功率
- 带速风险

### 滚珠丝杠

设计师通常已知：

- 负载质量
- 安装方向
- 导程
- 行程
- 目标速度
- 加速时间
- 丝杠效率
- 支撑方式
- 预期寿命
- 安全系数

输出：

- 轴向负载
- 驱动转矩
- 需求转速
- 临界转速校核
- 寿命校核
- 屈曲/支撑风险

### 直线导轨

设计师通常已知：

- 负载质量
- 滑块数量
- 安装姿态
- 负载重心偏置
- 行程
- 速度
- 加速度
- 期望寿命
- 安全系数

输出：

- 单滑块等效载荷
- 静载安全系数
- 动额定寿命需求
- 力矩风险
- 安装姿态风险

### 滚动轴承

设计师通常已知：

- 径向载荷和轴向载荷
- 轴承转速
- X/Y 载荷系数或轴承类型经验系数
- 工况系数
- 目标寿命小时
- 候选轴承动额定载荷 C
- 候选轴承静额定载荷 C0
- 寿命指数，球轴承取 3，滚子轴承取 10/3
- 安全系数

输出：

- 等效动载荷
- L10 寿命和寿命小时
- 所需动额定载荷
- 动载余量
- 静载余量
- 寿命和静载风险

### 直线轴承

设计师通常已知：

- 总径向载荷
- 共同承载的轴承数量
- 安装方向和受力不均修正
- 冲击系数
- 轴径
- 运行速度
- 目标行走寿命
- 候选直线轴承动额定载荷 C
- 安全系数

输出：

- 单轴承设计载荷
- 载荷余量
- 额定行走寿命
- 所需动额定载荷
- 速度指标和高速风险

### 联轴器

设计师通常已知：

- 电机额定扭矩
- 峰值扭矩
- 候选联轴器额定扭矩
- 冲击系数和温度修正系数
- 工作转速
- 负载惯量比
- 平行偏差、角向偏差和轴向位移
- 安全系数

输出：

- 额定修正扭矩
- 峰值修正扭矩
- 设计扭矩
- 扭矩余量
- 扭转需求指标
- 偏差指标
- 类型建议和偏差补偿风险

### 普通电机 / 调速电机

设计师通常已知：

- 负载质量
- 驱动轮或滚筒直径
- 目标线速度
- 启动加速时间
- 摩擦系数
- 外部阻力
- 垂直负载系数
- 传动效率
- 安全系数

输出：

- 摩擦力
- 加速力
- 垂直负载力
- 等效推力
- 输出转矩
- 需求转速
- 需求功率
- 调速范围风险

### 电磁阀 / 气动控制

设计师通常已知：

- 同时动作的气缸数量
- 气缸缸径、活塞杆直径和行程
- 阀到气缸的管路内径和长度
- 工作压力
- 循环时间和动作频率
- 候选阀额定流量
- 安全系数

输出：

- 伸出腔、回程腔和管路容积
- 单循环自由空气耗量
- 峰值流量需求
- 持续耗气量
- 阀额定流量余量
- 阀口径、管径和节拍风险

### 凸轮分割器

设计师通常已知：

- 工位数
- 单工位循环时间
- 分割时间
- 分割角度
- 运动曲线系数
- 转盘和工装折算惯量
- 外部负载扭矩
- 传动效率
- 安全系数

输出：

- 输出转速
- 停歇时间
- 峰值角速度
- 角加速度
- 惯量扭矩
- 设计扭矩
- 峰值功率
- 冲击和停歇余量风险

## 4. 下一步公式补强顺序

1. V 带
2. 链条
3. 齿轮
4. 减速机 / 直线模组
5. 手指气缸 / 滑台气缸
6. 机器人、拖链、传感器、材料、机加工等规则型模块
