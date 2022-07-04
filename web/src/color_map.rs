const colormap_BGR_1: [u32; 256] = [
    28, 65564, 131101, 196638, 327711, 393504, 459040, 524577, 655650, 721187, 786980, 918052,
    983589, 1049126, 1114663, 1245992, 1311528, 1442601, 1573930, 1770794, 1967658, 2164523,
    2361387, 2558252, 2755116, 2951980, 3083309, 3279917, 3476782, 3673646, 3870511, 4067375,
    4264239, 4461104, 4657968, 4854832, 5051696, 5248816, 5511216, 5708079, 5904943, 6101807,
    6298671, 6561327, 6758191, 6955054, 7151918, 7414318, 7611438, 7808302, 8005166, 8202029,
    8399149, 8596012, 8727595, 8924458, 9056042, 9253161, 9384488, 9581607, 9712935, 9910054,
    0x993925, 0x9c3a24, 0x9f3c24, 0xa13d23, 0xa43f22, 0xa64121, 0xa84221, 0xaa4420, 0xab461f,
    0xad481e, 0xae4a1d, 0xaf4c1c, 0xb04e1b, 0xb15019, 0xb35218, 0xb45417, 0xb55616, 0xb65815,
    0xb75a14, 0xb95c13, 0xba5e12, 0xbb6011, 0xbd6210, 0xbd640f, 0xbe660e, 0xbd680e, 0xbd6a0d,
    0xbd6d0c, 0xbd6f0c, 0xbc710b, 0xbc730a, 0xbc7509, 0xbc7709, 0xbc7908, 0xbb7b07, 0xbb7d06,
    0xbb7f06, 0xbb8205, 0xbb8404, 0xbb8604, 0xba8803, 0xb98a03, 0xb88c03, 0xb78d03, 0xb58f04,
    0xb49104, 0xb39304, 0xb29504, 0xb09605, 0xaf9805, 0xae9a05, 0xad9c05, 0xab9e06, 0xaa9f06,
    0xa9a106, 0xa8a306, 0xa6a506, 0xa5a707, 0xa4a808, 0xa2a909, 0xa0aa0b, 0x9fac0d, 0x9dad0e,
    0x9bae10, 0x9aaf11, 0x98b113, 9875988, 9810710, 9679895, 9549337, 9484058, 9353244, 9222429,
    9157407, 9026593, 8895523, 8830245, 8699432, 8633899, 8503086, 8437808, 8306739, 8241461,
    8110392, 8045115, 7914301, 7848768, 7717955, 7652677, 7521608, 7390795, 7325261, 7259985,
    7194452, 7128919, 7063642, 6998109, 6932577, 6867044, 6736231, 6670698, 6605165, 6539633,
    6474356, 6408823, 6343290, 6277757, 6212481, 6146948, 6081415, 6081418, 6081677, 6016144,
    6016147, 5950614, 5950873, 5885340, 5885343, 5819810, 5820069, 5820072, 5754539, 5754542,
    5689265, 5689268, 5689271, 5689529, 5689531, 5689790, 5689792, 5690050, 5690052, 5755847,
    5755849, 5756107, 5756109, 5756367, 5821906, 5822164, 5822422, 5822425, 5822683, 5888221,
    5888478, 5954272, 6020065, 6020322, 6086116, 6151653, 6217446, 6217704, 6283497, 6349290,
    6349548, 6415341, 6481134, 6546672, 6546929, 6612722, 6678516, 6744308, 6810101, 6875894,
    6941686, 7007735, 7073528, 7139320, 7205113, 7270905, 7402234, 7468027, 7533819, 7599612,
    7665405, 7731453, 7797246, 7862782, 7928575, 7994111, 8125439, 8190974, 8256510, 8322046,
    8387839, 8453375, 8518911, 8650238, 8715774, 8781310, 8846847, 8912639, 8978175, 9109246,
    9109246,
];
const colormap_BGR_2: [u32; 256] = [
    10453981, 10585053, 10650588, 10781659, 10912731, 10978522, 11109593, 11240665, 11306200,
    11437271, 11503062, 11634133, 11765204, 11830995, 11962066, 12027601, 12158928, 12224462,
    12355789, 12421324, 12552650, 12618185, 12749256, 12815046, 12880581, 13011907, 13077441,
    13143232, 13274558, 13340092, 13405883, 13471417, 13537207, 13668277, 13734067, 13799601,
    13865391, 13931181, 13996715, 14062505, 14128039, 14193829, 14259618, 14259616, 14325406,
    14390940, 14456729, 14456983, 14522516, 14588306, 14588560, 14654093, 14654346, 14719880,
    14720133, 14720131, 14785920, 14786173, 14786170, 14786424, 14786421, 14852210, 14852207,
    14852460, 14852457, 14852710, 14852707, 14787424, 14787421, 14787674, 14787671, 14722388,
    14722385, 14722638, 14657098, 14657351, 14591811, 14591808, 14526524, 14526521, 14461237,
    14395697, 14395693, 14330409, 14264868, 14199328, 14134043, 14068501, 14002958, 13937669,
    13937664, 13872384, 13806848, 13741568, 13676288, 13610752, 13611008, 13479936, 13414400,
    13349120, 13283584, 13218048, 13086976, 13021696, 12956160, 12825088, 12759552, 12628480,
    12562944, 12431872, 12300800, 12169472, 12103936, 11972864, 11841792, 11710464, 11579392,
    11448325, 11317261, 11251731, 11120664, 11055133, 10924065, 10792997, 10727464, 10596396,
    10465327, 10399794, 10268725, 10137656, 10006587, 9941053, 9809728, 9678659, 9613125, 9482056,
    9416522, 9285453, 9154383, 9088850, 8957524, 8826454, 8760921, 8629851, 8564317, 8433247,
    8367457, 8236388, 8170854, 8039784, 7973994, 7842924, 7777390, 7711856, 7580530, 7514996,
    7449462, 7318392, 7252602, 7187068, 7121534, 6990208, 6924674, 6859140, 6793349, 6727815,
    6662281, 6596491, 6530957, 6465167, 6399632, 6399634, 6333844, 6268310, 6202520, 6202521,
    6136731, 6071197, 6070943, 6005408, 6005410, 5939620, 5939621, 5939367, 5873832, 5873578,
    5873580, 5873325, 5873327, 5873072, 5872818, 5872819, 5872565, 5872566, 5872312, 5872313,
    5937595, 5937596, 5937342, 6002879, 6002624, 6002370, 6067907, 6133188, 6133189, 6198471,
    6198472, 6263753, 6329034, 6394571, 6394317, 6459854, 6525135, 6590672, 6655953, 6721234,
    6786771, 6852052, 6917589, 6982869, 7048406, 7179223, 7244760, 7310041, 7375577, 7440858,
    7571931, 7637467, 7702748, 7833820, 7899101, 8030173, 8095454, 8160990, 8292063, 8357343,
    8488415, 8553951, 8685024, 8750304, 8881376, 8946912, 9077984, 9208800, 9274336, 9405408,
    9470944, 9602016, 9733088, 9798623, 9929695, 10060767, 10126302, 10257374, 10388446, 10453981,
];

pub fn color_map_to_u8(color_map: &[u32; 256]) -> [u8; 1024] {
    let u8_arr: Vec<u8> = color_map
        .into_iter()
        .map(|x| to_rgba(*x))
        .flatten()
        .collect();
    let mut v = [0_u8; 1024];
    for i in 0..v.len() {
        v[i] = u8_arr[i];
    }
    return v;
}
pub fn colormap1() -> [u8; 1024] {
    color_map_to_u8(&colormap_BGR_1)
}

pub fn colormap2() -> [u8; 1024] {
    color_map_to_u8(&colormap_BGR_2)
}

const fn to_rgba(rgba: u32) -> [u8; 4] {
    let r = ((rgba >> 0) & 0xff) as u8;
    let g = ((rgba >> 8) & 0xff) as u8;
    let b = ((rgba >> 16) & 0xff) as u8;
    let a = 255;
    [r, g, b, a]
}
