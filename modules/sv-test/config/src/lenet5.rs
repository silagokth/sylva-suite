use sv_lib::model::*;


pub fn lenet5(db: &mut DataBase) -> Result<(), Box <dyn std::error::Error>> {
 
    db.global_constraint.max_energy = 10000;
    db.global_constraint.max_width = 10000;
    db.global_constraint.max_height = 10000;
    db.global_constraint.max_latency = 100000;
    db.global_constraint.max_period = 100000;

    db.hyper_parameter.bind_w_area = 1;
    db.hyper_parameter.bind_w_energy = 1;
    db.hyper_parameter.bind_w_latency = 1;
    db.hyper_parameter.bind_relaxation_factor = 1.1;
    db.hyper_parameter.place_relaxation_factor = 1.5;
    db.hyper_parameter.place_reserved_routing_size = 1;

    let prefix = "examples/lenet5";


    let load_input_output_pattern = vec![3, 3, 1, 6, 5, 15, 25, 27, 25, 23, 23, 28, 28, 28, 33, 37, 38, 37, 39, 44, 45, 49, 53, 58, 62, 69, 73, 82, 87, 92, 94, 94, 101, 103, 107, 110, 110, 111, 117, 115, 125, 123, 121, 119, 129, 135, 136, 144, 147, 154, 159, 168, 172, 173, 176, 185, 195, 199, 200, 210, 219, 224, 233, 237];
    let conv1_input_pattern = vec![6, 15, 15, 21, 23, 24, 30, 39, 39, 47, 47, 53, 55, 59, 68, 73, 73, 80, 82, 85, 85, 84, 92, 97, 107, 106, 108, 108, 107, 106, 106, 105, 107, 109, 115, 116, 119, 117, 117, 121, 121, 120, 119, 122, 130, 128, 127, 132, 136, 142, 151, 153, 154, 157, 155, 164, 170, 169, 171, 169, 175, 179, 180, 178];
    let conv1_output_pattern = vec![0, 1, 1, 10, 16, 19, 26, 27, 37, 36, 37, 40, 49, 59, 65, 66, 64, 65, 63, 73, 83, 86, 88, 89, 95, 98, 105, 105, 103, 102, 104, 109, 114, 113, 118, 124, 123, 130, 135, 140, 142, 140, 143, 146, 154, 159, 166, 168, 172, 175, 178, 179, 187, 194, 195, 198, 203, 210, 212, 219, 219, 228, 237, 247, 255, 261, 263, 262, 269, 270, 270, 280, 281, 291, 295, 302, 304, 310, 316, 326, 329, 328, 330, 332, 333, 333, 334, 338, 337, 335, 336, 336, 345, 353, 351, 357, 366, 373, 381, 386, 395, 401, 410, 418, 424, 422, 432, 442, 447, 451, 456, 462, 470, 471, 479, 483, 481, 483, 488, 487, 493, 503, 502, 512, 519, 523, 521, 522, 523, 533, 532, 530, 533, 533, 538, 543, 550, 559, 565, 573, 571, 573, 574, 584, 587, 595, 602, 606, 609, 616, 624, 628, 634, 638, 640, 641, 643, 642, 644, 649, 658, 666, 671, 676, 684, 692, 695, 704, 711, 719, 720, 720, 722, 731, 736, 745, 751, 761, 764, 770, 771, 779, 786, 790, 800, 806, 814, 814, 821, 820, 820, 819, 820, 821, 827, 837, 837, 845, 844, 845, 846, 848, 856, 858, 858, 858, 858, 860, 864, 866, 874, 873, 873, 882, 890, 889, 897, 899, 905, 906, 916, 916, 924, 928, 929, 928, 937, 943, 947, 950, 948, 948, 956, 957, 960, 959, 957, 964, 970, 980, 984, 993, 1000, 1008, 1013, 1016, 1025, 1025, 1032, 1038, 1036, 1042, 1046, 1045, 1054, 1052, 1052, 1058, 1061, 1063, 1063, 1068, 1071, 1073, 1083, 1088, 1097, 1105, 1105, 1112, 1115, 1118, 1128, 1131, 1130, 1140, 1141, 1148, 1151, 1149, 1154, 1156, 1166, 1171, 1171, 1177, 1185, 1185, 1190, 1188, 1196, 1203, 1201, 1211, 1218, 1218, 1223, 1232, 1233, 1233, 1231, 1229, 1232, 1234, 1232, 1232, 1234, 1244, 1242, 1244, 1243, 1243, 1245, 1246, 1256, 1265, 1272, 1280, 1289, 1294, 1302, 1306, 1311, 1314, 1323, 1328, 1338, 1347, 1357, 1363, 1363, 1368, 1378, 1385, 1384, 1392];
    let pooling1_input_pattern = vec![0, 7, 17, 16, 21, 24, 33, 42, 52, 62, 64, 64, 65, 75, 83, 91, 98, 107, 111, 114, 123, 133, 142, 142, 141, 146, 148, 147, 150, 158, 165, 165, 173, 172, 177, 185, 192, 199, 202, 202, 203, 204, 207, 215, 222, 220, 223, 226, 225, 232, 238, 242, 249, 250, 256, 260, 269, 274, 278, 280, 282, 288, 297, 297, 307, 313, 319, 320, 320, 330, 332, 337, 342, 340, 345, 354, 355, 359, 367, 372, 381, 380, 379, 380, 386, 393, 401, 411, 421, 426, 430, 432, 438, 436, 437, 436, 436, 434, 440, 440, 450, 456, 463, 461, 469, 479, 482, 480, 488, 486, 496, 504, 502, 508, 508, 515, 515, 515, 514, 513, 518, 521, 524, 530, 530, 531, 536, 546, 551, 550, 551, 560, 564, 572, 579, 583, 592, 593, 600, 609, 609, 613, 616, 614, 614, 618, 624, 630, 628, 629, 637, 647, 656, 665, 671, 675, 676, 678, 684, 689, 698, 708, 709, 711, 717, 723, 732, 737, 740, 748, 756, 766, 766, 772, 771, 774, 774, 779, 789, 787, 787, 796, 802, 811, 811, 816, 820, 825, 825, 823, 831, 831, 839, 849, 847, 856, 863, 865, 875, 875, 883, 886, 895, 897, 902, 909, 918, 928, 932, 934, 942, 943, 947, 946, 955, 964, 965, 966, 966, 973, 982, 981, 981, 990, 996, 998, 996, 994, 993, 998, 1001, 1006, 1010, 1009, 1008, 1016, 1018, 1025, 1034, 1035, 1033, 1032, 1039, 1044, 1043, 1053, 1059, 1065, 1073, 1072, 1081, 1081, 1090, 1097, 1096, 1098, 1104, 1105, 1107, 1109, 1111, 1114, 1113, 1123, 1129, 1139, 1138, 1142, 1149, 1156, 1164, 1168, 1177, 1177, 1177, 1178, 1185, 1192, 1197, 1196, 1197, 1196, 1196, 1198, 1208, 1213, 1215, 1223, 1226, 1226, 1236, 1236, 1236, 1236, 1236, 1238, 1243, 1251, 1257, 1258, 1257, 1257, 1258, 1266, 1275, 1280, 1284, 1291, 1295, 1297, 1299, 1300, 1302, 1305, 1310, 1312, 1318, 1328, 1328, 1327, 1335, 1344, 1349, 1355, 1362, 1371, 1375, 1382, 1384, 1383, 1381, 1381, 1379, 1380, 1378, 1379];
    let pooling1_output_pattern = vec![7, 12, 10, 9, 17, 27, 28, 34, 40, 40, 41, 44, 45, 50, 58, 65, 65, 71, 79, 80, 86, 87, 92, 101, 107, 116, 126, 134, 138, 144, 142, 152, 158, 161, 171, 179, 179, 187, 197, 200, 208, 208, 213, 220, 227, 229, 234, 237, 238, 245, 253, 259, 268, 269, 277, 277, 285, 292, 296, 306, 306, 309, 311, 314, 323, 331, 334, 335, 335, 338, 344, 348, 347, 357, 359, 368, 366, 371, 372, 376, 381, 379, 384, 394];
    let conv2_input_pattern = vec![8, 9, 8, 6, 5, 10, 12, 12, 14, 23, 32, 40, 47, 52, 55, 58, 65, 64, 67, 74, 78, 86, 93, 99, 98, 104, 112, 120, 130, 134, 139, 141, 144, 145, 151, 156, 166, 176, 179, 188, 193, 203, 211, 214, 213, 220, 218, 226, 228, 234, 235, 242, 244, 243, 242, 250, 258, 265, 269, 271, 281, 289, 287, 297, 307, 309, 317, 325, 323, 327, 332, 341, 342, 342, 349, 348, 349, 356, 365, 365, 364, 366, 368, 368];
    let conv2_output_pattern = vec![10, 10, 17, 27, 33, 35, 44, 49, 47, 48, 52, 51, 54, 63, 68, 74, 73, 76, 79, 83, 91, 97, 100, 103, 107, 116, 119, 124, 125, 123, 125, 132, 141, 145, 155, 162, 160, 162, 168, 173, 180, 181, 179, 183, 193, 195, 193, 192, 196, 202, 205, 204, 209, 219, 225, 225, 224, 230, 237, 236, 240, 247, 250, 250, 255, 253, 262, 272, 279, 285, 293, 297, 297, 302, 309, 315, 314, 316, 326, 334, 342, 352, 359, 359, 361, 367, 375, 383, 384, 385, 390, 399, 405, 412, 421, 424, 423, 421, 419, 419, 422, 423, 429, 438, 443, 447, 455, 456, 459, 460, 462, 460, 460, 466, 476, 478, 488, 486, 484, 490, 498, 504, 509, 511, 517, 519, 527, 535, 534, 540, 547, 556, 554, 564, 566, 567, 573, 578, 578, 585, 589, 591, 591, 594, 604, 609, 610, 615, 614, 612, 610, 617, 621, 628, 628, 638, 641, 641, 645, 647];
    let pooling2_input_pattern = vec![2, 11, 17, 27, 37, 44, 47, 45, 43, 43, 46, 53, 58, 67, 68, 70, 68, 77, 85, 92, 100, 98, 102, 102, 102, 105, 109, 112, 111, 112, 122, 131, 129, 139, 147, 156, 159, 160, 162, 163, 165, 173, 181, 190, 197, 204, 205, 205, 215, 213, 211, 212, 217, 227, 234, 238, 242, 249, 259, 258, 258, 258, 261, 260, 268, 266, 276, 279, 282, 284, 282, 287, 289, 297, 302, 301, 299, 302, 301, 304, 309, 311, 316, 319, 319, 329, 332, 341, 347, 345, 353, 358, 367, 372, 374, 381, 387, 392, 396, 395, 396, 403, 406, 404, 403, 412, 416, 417, 426, 431, 434, 432, 440, 446, 455, 456, 456, 457, 465, 474, 478, 484, 494, 504, 512, 512, 514, 518, 520, 527, 533, 543, 548, 551, 558, 559, 569, 578, 584, 587, 586, 587, 588, 593, 591, 592, 591, 598, 605, 607, 605, 615, 623, 628, 630, 630, 634, 635, 640, 639];
    let pooling2_output_pattern = vec![6, 4, 10, 17, 24, 23, 23, 23, 30, 37, 36, 40, 43, 50, 56, 56, 57, 63, 73, 79, 79, 84, 85, 83, 86, 91, 99, 101, 110, 112, 116, 114, 112, 119, 128, 136, 143, 151, 151, 155, 154, 160, 165, 168, 175, 182, 181, 184, 186, 191, 198, 206, 208, 215, 223, 229, 236, 241, 247, 257, 267, 276, 279, 284, 288, 297, 305, 305, 315, 320, 325, 332, 340, 343, 353, 360, 363, 368, 377, 387];
    let conv3_input_pattern = vec![6, 14, 20, 30, 40, 40, 42, 51, 51, 53, 63, 64, 67, 76, 76, 81, 90, 93, 102, 100, 103, 106, 110, 111, 120, 121, 126, 136, 138, 148, 146, 148, 152, 159, 158, 160, 160, 161, 167, 174, 173, 181, 181, 189, 197, 207, 208, 209, 211, 209, 209, 209, 216, 221, 229, 237, 235, 233, 242, 249, 249, 256, 263, 270, 274, 276, 275, 273, 274, 273, 282, 280, 278, 277, 287, 294, 304, 312, 315, 320];
    let conv3_output_pattern = vec![10, 13, 16, 15, 13, 13, 17, 21, 21, 26, 32, 30, 28, 27, 29, 39, 41, 47, 51, 55, 65, 74, 76, 79, 86, 89, 91, 91, 101, 107, 111, 119, 127, 134, 135, 136, 142, 147, 148, 146, 153, 159, 167, 175, 177, 179, 184, 194, 203, 201, 200, 209, 213, 217, 216, 224, 223, 229, 228, 230, 233, 242, 248, 258, 264, 274, 280, 286, 289, 296, 300, 306, 304, 314, 315, 323, 330, 340, 339, 349, 355, 353, 351, 361, 370, 369, 374, 377, 380, 383, 386, 384, 389, 393, 395, 393, 392, 392, 401, 408, 410, 408, 415, 416, 423, 432, 430, 438, 444, 443, 451, 453, 457, 456, 465, 464, 464, 470, 479, 489];
    let reshape_input_pattern = vec![0, 0, 6, 11, 15, 23, 32, 40, 49, 55, 56, 65, 75, 81, 91, 94, 97, 103, 109, 110, 116, 119, 126, 136, 139, 144, 142, 152, 154, 152, 153, 159, 164, 173, 182, 180, 187, 185, 194, 199, 201, 199, 198, 198, 205, 211, 215, 216, 216, 225, 233, 243, 242, 250, 249, 255, 258, 264, 262, 267, 269, 271, 278, 286, 286, 290, 295, 299, 301, 303, 305, 312, 320, 328, 327, 325, 325, 332, 331, 340, 349, 352, 353, 356, 359, 368, 369, 371, 369, 378, 377, 377, 381, 385, 386, 384, 382, 384, 389, 394, 404, 410, 410, 415, 418, 419, 421, 429, 430, 440, 445, 455, 457, 462, 470, 479, 480, 488, 486, 488];
    let reshape_output_pattern = vec![2, 9, 17, 22, 22, 30, 35, 42];
    let fc1_input_pattern = vec![0, 3, 11, 14, 13, 17, 19, 25];
    let fc1_output_pattern = vec![8, 12, 19, 28, 30, 31];
    let fc2_input_pattern = vec![4, 7, 13, 17, 27, 34];
    let fc2_output_pattern = vec![0];
    let store_output_input_pattern = vec![3];
    

    let load_input_output_token = load_input_output_pattern.len() as i32;
    let conv1_input_token = conv1_input_pattern.len() as i32;
    let conv1_output_token = conv1_output_pattern.len() as i32;
    let pooling1_input_token = pooling1_input_pattern.len() as i32;
    let pooling1_output_token = pooling1_output_pattern.len() as i32;
    let conv2_input_token = conv2_input_pattern.len() as i32;
    let conv2_output_token = conv2_output_pattern.len() as i32;
    let pooling2_input_token = pooling2_input_pattern.len() as i32;
    let pooling2_output_token = pooling2_output_pattern.len() as i32;
    let conv3_input_token = conv3_input_pattern.len() as i32;
    let conv3_output_token = conv3_output_pattern.len() as i32;
    let reshape_input_token = reshape_input_pattern.len() as i32;
    let reshape_output_token = reshape_output_pattern.len() as i32;
    let fc1_input_token = fc1_input_pattern.len() as i32;
    let fc1_output_token = fc1_output_pattern.len() as i32;
    let fc2_input_token = fc2_input_pattern.len() as i32;
    let fc2_output_token = fc2_output_pattern.len() as i32;
    let store_output_input_token = store_output_input_pattern.len() as i32;


    fn to_pair_list(data: Vec<i32>) -> Vec<PairIntInt> {
        data.into_iter().enumerate()
            .map(|(i, val)| PairIntInt {
                key: i as i32,
                value: val,
            })
            .collect()
    }


    fn cal_max(d1: &Vec<i32>, d2: &Vec<i32>) -> i32 {
        let mut combined = d1.clone();
        combined.extend(d2);
        combined.iter().max().map(|v| v + 1).unwrap_or(0)
    }


    db.alimp_lib.entries.push(AlimpEntry {
        func: "input_32x32".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 5, 
                energy: 10, 
                latency: cal_max(&load_input_output_pattern, &vec![]),
                input_addr_time_patterns: vec![],
                output_addr_time_patterns: to_pair_list(load_input_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "conv_32x32_5x5".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 2, 
                height: 2, 
                energy: 10, 
                latency: cal_max(&conv1_input_pattern, &conv1_output_pattern),
                input_addr_time_patterns: to_pair_list(conv1_input_pattern),
                output_addr_time_patterns: to_pair_list(conv1_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "max_pool_28x28_2".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 10, 
                height: 10, 
                energy: 10, 
                latency: cal_max(&pooling1_input_pattern, &pooling1_output_pattern),
                input_addr_time_patterns: to_pair_list(pooling1_input_pattern),
                output_addr_time_patterns: to_pair_list(pooling1_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "conv_14x14_5x5".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 10, 
                height: 10, 
                energy: 10, 
                latency: cal_max(&conv2_input_pattern, &conv2_output_pattern),
                input_addr_time_patterns: to_pair_list(conv2_input_pattern),
                output_addr_time_patterns: to_pair_list(conv2_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "max_pool_10x10_2".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 10, 
                height: 10, 
                energy: 10, 
                latency: cal_max(&pooling2_input_pattern, &pooling2_output_pattern),
                input_addr_time_patterns: to_pair_list(pooling2_input_pattern),
                output_addr_time_patterns: to_pair_list(pooling2_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "conv_5x5_5x5".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 5, 
                energy: 10, 
                latency: cal_max(&conv3_input_pattern, &conv3_output_pattern),
                input_addr_time_patterns: to_pair_list(conv3_input_pattern),
                output_addr_time_patterns: to_pair_list(conv3_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "reshape_5x5_1".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 5, 
                energy: 10, 
                latency: cal_max(&reshape_input_pattern, &reshape_output_pattern),
                input_addr_time_patterns: to_pair_list(reshape_input_pattern),
                output_addr_time_patterns: to_pair_list(reshape_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "dense_120_84".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 1, 
                energy: 10, 
                latency: cal_max(&fc1_input_pattern, &fc1_output_pattern),
                input_addr_time_patterns: to_pair_list(fc1_input_pattern),
                output_addr_time_patterns: to_pair_list(fc1_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "dense_84_10".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 1, 
                energy: 10, 
                latency: cal_max(&fc2_input_pattern, &fc2_output_pattern),
                input_addr_time_patterns: to_pair_list(fc2_input_pattern),
                output_addr_time_patterns: to_pair_list(fc2_output_pattern), 
                ..Default::default()
            },
        ],
    });

    db.alimp_lib.entries.push(AlimpEntry {
        func: "output_10".to_string(),
        instances: vec![
            AlimpInstance { 
                width: 5, 
                height: 1, 
                energy: 10, 
                latency: cal_max(&store_output_input_pattern, &vec![]),
                input_addr_time_patterns: to_pair_list(store_output_input_pattern),
                output_addr_time_patterns: vec![], 
                ..Default::default()
            },
        ],
    });

    db.app_graph.nodes.push(AppNode {
        id: "load_input".to_string(),
        func: "input_32x32".to_string(),
        executable: format!("{}/load-input --addr 0 --size 64", prefix),
        output_ports: vec![
            AppNodePort {
                id: "load_input_out".to_string(),
                rate: 1,
                token_size: load_input_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "conv1".to_string(),
        func: "conv_32x32_5x5".to_string(),
        executable: format!("{}/conv --input_image_channel 1 --input_image_size 32 --kernel_channel 6 --kernel_size 5 --stride 1 --kernel {}/data/conv1_kernel.json --bias {}/data/conv1_bias.json", prefix, prefix, prefix),
        input_ports: vec![
            AppNodePort {
                id: "conv1_in".to_string(),
                rate: 1,
                token_size: conv1_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "conv1_out".to_string(),
                rate: 1,
                token_size: conv1_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "pooling1".to_string(),
        func: "max_pool_28x28_2".to_string(),
        executable: format!("{}/pooling --input_image_channel 6 --input_image_size 28 --kernel_size 2 --stride 2 --mode average", prefix),
        input_ports: vec![
            AppNodePort {
                id: "pooling1_in".to_string(),
                rate: 1,
                token_size: pooling1_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "pooling1_out".to_string(),
                rate: 1,
                token_size: pooling1_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "conv2".to_string(),
        func: "conv_14x14_5x5".to_string(),
        executable: format!("{}/conv --input_image_channel 6 --input_image_size 14 --kernel_channel 16 --kernel_size 5 --stride 1 --kernel {}/data/conv2_kernel.json --bias {}/data/conv2_bias.json", prefix, prefix, prefix),
        input_ports: vec![
            AppNodePort {
                id: "conv2_in".to_string(),
                rate: 1,
                token_size: conv2_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "conv2_out".to_string(),
                rate: 1,
                token_size: conv2_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "pooling2".to_string(),
        func: "max_pool_10x10_2".to_string(),
        executable: format!("{}/pooling --input_image_channel 16 --input_image_size 10 --kernel_size 2 --stride 2 --mode average", prefix),
        input_ports: vec![
            AppNodePort {
                id: "pooling2_in".to_string(),
                rate: 1,
                token_size: pooling2_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "pooling2_out".to_string(),
                rate: 1,
                token_size: pooling2_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "conv3".to_string(),
        func: "conv_5x5_5x5".to_string(),
        executable: format!("{}/conv --input_image_channel 16 --input_image_size 5 --kernel_channel 120 --kernel_size 5 --stride 1 --kernel {}/data/conv3_kernel.json --bias {}/data/conv3_bias.json", prefix, prefix, prefix),
        input_ports: vec![
            AppNodePort {
                id: "conv3_in".to_string(),
                rate: 1,
                token_size: conv3_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "conv3_out".to_string(),
                rate: 1,
                token_size: conv3_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "reshape".to_string(),
        func: "reshape_5x5_1".to_string(),
        executable: format!("{}/reshape --input_row 120 --input_col 1 --output_row 1 --output_col 120", prefix),
        input_ports: vec![
            AppNodePort {
                id: "reshape_in".to_string(),
                rate: 1,
                token_size: reshape_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "reshape_out".to_string(),
                rate: 1,
                token_size: reshape_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "fc1".to_string(),
        func: "dense_120_84".to_string(),
        executable: format!("{}/fc --input_size 120 --output_size 84 --weight {}/data/fc1_weight.json --bias {}/data/fc1_bias.json --activation=tanh", prefix, prefix, prefix),
        input_ports: vec![
            AppNodePort {
                id: "fc1_in".to_string(),
                rate: 1,
                token_size: fc1_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "fc1_out".to_string(),
                rate: 1,
                token_size: fc1_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "fc2".to_string(),
        func: "dense_84_10".to_string(),
        executable: format!("{}/fc --input_size 84 --output_size 10 --weight {}/data/fc2_weight.json --bias {}/data/fc2_bias.json --activation=softmax", prefix, prefix, prefix),
        input_ports: vec![
            AppNodePort {
                id: "fc2_in".to_string(),
                rate: 1,
                token_size: fc2_input_token,
                ..Default::default()
            },
        ],
        output_ports: vec![
            AppNodePort {
                id: "fc2_out".to_string(),
                rate: 1,
                token_size: fc2_output_token,
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    db.app_graph.nodes.push(AppNode {
        id: "store_output".to_string(),
        func: "output_10".to_string(),
        executable: format!("{}/store-output --addr 64 --size 1", prefix),
        input_ports: vec![
            AppNodePort {
                id: "store_output_in".to_string(),
                rate: 1,
                token_size: store_output_input_token,
                ..Default::default()
            },
        ],
        ..Default::default() 
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_load_input_conv1".to_string(),
        source_node: "load_input".to_string(),
        target_node: "conv1".to_string(),
        source_port: "load_input_out".to_string(),
        target_port: "conv1_in".to_string(),
        token_size: load_input_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_conv1_pooling1".to_string(),
        source_node: "conv1".to_string(),
        target_node: "pooling1".to_string(),
        source_port: "conv1_out".to_string(),
        target_port: "pooling1_in".to_string(),
        token_size: conv1_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_pooling1_conv2".to_string(),
        source_node: "pooling1".to_string(),
        target_node: "conv2".to_string(),
        source_port: "pooling1_out".to_string(),
        target_port: "conv2_in".to_string(),
        token_size: pooling1_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_conv2_pooling2".to_string(),
        source_node: "conv2".to_string(),
        target_node: "pooling2".to_string(),
        source_port: "conv2_out".to_string(),
        target_port: "pooling2_in".to_string(),
        token_size: conv2_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_pooling2_conv3".to_string(),
        source_node: "pooling2".to_string(),
        target_node: "conv3".to_string(),
        source_port: "pooling2_out".to_string(),
        target_port: "conv3_in".to_string(),
        token_size: pooling2_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_conv3_reshape".to_string(),
        source_node: "conv3".to_string(),
        target_node: "reshape".to_string(),
        source_port: "conv3_out".to_string(),
        target_port: "reshape_in".to_string(),
        token_size: conv3_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_reshape_fc1".to_string(),
        source_node: "reshape".to_string(),
        target_node: "fc1".to_string(),
        source_port: "reshape_out".to_string(),
        target_port: "fc1_in".to_string(),
        token_size: reshape_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_fc1_fc2".to_string(),
        source_node: "fc1".to_string(),
        target_node: "fc2".to_string(),
        source_port: "fc1_out".to_string(),
        target_port: "fc2_in".to_string(),
        token_size: fc1_output_token,
        ..Default::default()
    });

    db.app_graph.edges.push(AppEdge {
        id: "edge_fc2_store_output".to_string(),
        source_node: "fc2".to_string(),
        target_node: "store_output".to_string(),
        source_port: "fc2_out".to_string(),
        target_port: "store_output_in".to_string(),
        token_size: fc2_output_token,
        ..Default::default()
    });

    db.app_graph.global_mem_image = format!("{}/mem/global_mem_image.json", prefix);
    db.app_graph.global_mem_reference = format!("{}/mem/global_mem_reference.json", prefix);

    Ok(())
}
