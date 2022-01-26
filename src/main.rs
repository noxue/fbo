use core::slice;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem;

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    let path = match args.next() {
        Some(v) => v,
        None => {
            let res = Res {
                code: 0,
                msg: format!("缺少要解析的二进制文件，用法:{} 二进制文件路径", exec),
                data: None,
            };
            println!("{}", serde_json::to_string(&res).unwrap());
            return;
        }
    };

    let res = read_to_json(&path);

    println!("{}", serde_json::to_string(&res).unwrap());
}

fn read_to_json(path: &str) -> Res {
    let fp = match File::open(path) {
        Ok(v) => v,
        Err(e) => {
            return Res {
                code: 0,
                msg: e.to_string(),
                data: None,
            };
        }
    };
    let mut buffer = BufReader::new(fp);

    let mut pos = 0;

    let mut buf = Vec::new();
    match buffer.read_to_end(&mut buf) {
        Ok(v) => v,
        Err(e) => {
            return Res {
                code: 0,
                msg: e.to_string(),
                data: None,
            };
        }
    };

    // 获取头信息
    let mut fbo_file_inf: FboFileInf = unsafe { mem::zeroed() };
    let fbo_file_inf_slice =
        unsafe { slice::from_raw_parts_mut(&mut fbo_file_inf as *mut _ as *mut u8, 256) };
    fbo_file_inf_slice.copy_from_slice(&buf[pos..256]);
    pos += 256;
    // println!("fbo_file_inf:{:#?}", fbo_file_inf);

    // 保存封面信息和数据
    let mut data = Data {
        fbo_file_inf: fbo_file_inf.clone(),
        fbo_data_list: vec![],
    };

    // 每次读取的数据长度
    let len = mem::size_of::<FboData>() - 1000 + fbo_file_inf.test_start_inf.batt_sum as usize * 2;

    while pos + len < buf.len() {
        // 读取一次数据
        let mut fbo_data: FboData = unsafe { mem::zeroed() };

        let fbo_data_slice =
            unsafe { slice::from_raw_parts_mut(&mut fbo_data as *mut _ as *mut u8, len) };
        fbo_data_slice.copy_from_slice(&buf[pos..(pos + len)]);

        // 如果标志位不对，往后移动一个位置重新读取
        let data_type = &fbo_data.data_type;
        if !((data_type.type_tag_0 == data_type.type_tag_1
            && data_type.type_tag_1 == data_type.type_tag_2
            && data_type.type_tag_2 == data_type.type_tag_3)
            && (data_type.type_tag_0 == 0xfc || data_type.type_tag_0 == 0xfd))
        {
            pos += 1;
            continue;
        }

        pos += len;

        // 每次获取的数据添加到列表中
        let mut fbo_data_tag = FboDataTagJson {
            single_vol: vec![],
            crc16: fbo_data.fbo_data.crc16,
            m_test_time: fbo_data.fbo_data.m_test_time,
            batt_group: fbo_data.fbo_data.batt_group,
            batt_sum: fbo_data.fbo_data.batt_sum,
            online_vol: fbo_data.fbo_data.online_vol,
            sum_voltage: fbo_data.fbo_data.sum_voltage,
            sub_current: fbo_data.fbo_data.sub_current,
            all_cap: fbo_data.fbo_data.all_cap,
            sub_cap: fbo_data.fbo_data.sub_cap,
        };

        for i in 0..fbo_file_inf.test_start_inf.batt_sum {
            fbo_data_tag
                .single_vol
                .push(*fbo_data.fbo_data.single_vol.get(i as usize).unwrap());
        }

        let fbo_data_json = FboDataJson {
            data_type: fbo_data.data_type,
            fbo_data: fbo_data_tag,
        };
        data.fbo_data_list.push(fbo_data_json);
    }

    Res {
        code: 0,
        msg: "".to_string(),
        data: Some(data),
    }
}

//日期时间
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DateTime {
    year: u8, // 年（实际年份-2000）
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

//时间
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestTime {
    hour: u8,
    minute: u8,
    second: u8,
}

//数据头标记，连续的四个相同的值，FC表示充电，FD表示放电
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DataType {
    type_tag_0: u8,
    type_tag_1: u8,
    type_tag_2: u8,
    type_tag_3: u8,
}

#[repr(C)]
#[derive(Debug)]
struct FboDataTag {
    crc16: u16,             // 校验码
    m_test_time: TestTime,  // 测试时长
    batt_group: u8,         //电池组数
    batt_sum: u16,          // 电池节数
    online_vol: u16,        // 在线电压
    sum_voltage: u16,       // 组端电压
    sub_current: [u16; 4],  // 支路电流
    all_cap: u16,           // 测试容量
    sub_cap: [u16; 5],      // 支路测试容量
    single_vol: [u16; 500], // 单体电压
}
#[repr(C)]
#[derive(Debug)]
struct FboData {
    data_type: DataType,
    fbo_data: FboDataTag,
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DataStopInf {
    test_time_long: TestTime, // 测试时长
    stop_type: u8,            // 结束方式
    block_sum: u8,            // 保存数据的总块数
    stand_by: u8,             // 保留备用
    s_max_index: [u16; 4],    // 最高单体索引
    s_min_index: [u16; 4],    // 最低单体索引
    s_max_vol: [u16; 4],      // 最高单体
    s_min_vol: [u16; 4],      // 最低单体
    test_cap: u16,            // 测试容量
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DataStartInf {
    test_start_time: DateTime, // 放电开始的时间
    device: u8,                // 仪表类型
    data_version: u8,          // 数据版本
    data_type: u8,             // 数据类型;0xFD表示放电,0xFC表示充电
    hour_rate: u8,             // 小时率， 放电率
    save_interval: u8,         // 采集间隔
    monomer_vol: u8,           // 单体电压类型
    std_cap: u16,              // 标称容量
    test_cur: u16,             // 测试电流
    mvl_limit: u16,            // 单体下限
    sum_vl_limit: u16,         // 组端下限
    batt_sum: u16,             // 单体数量
    batt_group: u16,           // 电池组数
    mvl_limit_count: u16,      // 单体下限个数
}

//文件头结构
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct FboFileInf {
    test_start_inf: DataStartInf,
    test_stop_inf: DataStopInf,
}

// 用于json序列化
#[derive(Debug, Serialize, Deserialize, Clone)]
struct FboDataTagJson {
    crc16: u16,            // 校验码
    m_test_time: TestTime, // 测试时长
    batt_group: u8,        //电池组数
    batt_sum: u16,         // 电池节数
    online_vol: u16,       // 在线电压
    sum_voltage: u16,      // 组端电压
    sub_current: [u16; 4], // 支路电流
    all_cap: u16,          // 测试容量
    sub_cap: [u16; 5],     // 支路测试容量
    single_vol: Vec<u16>,  // 单体电压
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FboDataJson {
    data_type: DataType,
    fbo_data: FboDataTagJson,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Data {
    fbo_file_inf: FboFileInf,
    fbo_data_list: Vec<FboDataJson>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Res {
    code: i32,
    msg: String,
    data: Option<Data>,
}
