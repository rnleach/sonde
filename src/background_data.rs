use coords::TPCoords;

lazy_static! {
    pub static ref ISOBAR_PNTS: Vec<(TPCoords, TPCoords)> = {
        let mut v: Vec<(TPCoords, TPCoords)> = vec![];
        v.push(((-150.0,1050.0),(60.0,1050.0)));
        v.push(((-150.0,1000.0),(60.0,1000.0)));
        v.push(((-150.0,925.0),(60.0,925.0)));
        v.push(((-150.0,850.0),(60.0,850.0)));
        v.push(((-150.0,700.0),(60.0,700.0)));
        v.push(((-150.0,500.0),(60.0,500.0)));
        v.push(((-150.0,300.0),(60.0,300.0)));
        v.push(((-150.0,200.0),(60.0,200.0)));
        v.push(((-150.0,100.0),(60.0,100.0)));
        v
    };

    pub static ref COLD_ISOTHERM_PNTS: Vec<(TPCoords, TPCoords)> = {
        let mut v: Vec<(TPCoords, TPCoords)> = vec![];
        v.push(((-150.0,1050.0),(-150.0,99.0)));
        v.push(((-140.0,1050.0),(-140.0,99.0)));
        v.push(((-130.0,1050.0),(-130.0,99.0)));
        v.push(((-120.0,1050.0),(-120.0,99.0)));
        v.push(((-110.0,1050.0),(-110.0,99.0)));
        v.push(((-100.0,1050.0),(-100.0,99.0)));
        v.push(((-90.0,1050.0),(-90.0,99.0)));
        v.push(((-80.0,1050.0),(-80.0,99.0)));
        v.push(((-70.0,1050.0),(-70.0,99.0)));
        v.push(((-60.0,1050.0),(-60.0,99.0)));
        v.push(((-50.0,1050.0),(-50.0,99.0)));
        v.push(((-40.0,1050.0),(-40.0,99.0)));
        v.push(((-30.0,1050.0),(-30.0,99.0)));
        v.push(((-25.0,1050.0),(-25.0,99.0)));
        v.push(((-20.0,1050.0),(-20.0,99.0)));
        v.push(((-15.0,1050.0),(-15.0,99.0)));
        v.push(((-10.0,1050.0),(-10.0,99.0)));
        v.push(((-5.0,1050.0),(-5.0,99.0)));
        v.push(((0.0,1050.0),(0.0,99.0)));
        v
    };

    pub static ref WARM_ISOTHERM_PNTS: Vec<(TPCoords, TPCoords)> = {
        let mut v: Vec<(TPCoords, TPCoords)> = vec![];
        v.push(((5.0,1050.0),(5.0,99.0)));
        v.push(((10.0,1050.0),(10.0,99.0)));
        v.push(((15.0,1050.0),(15.0,99.0)));
        v.push(((20.0,1050.0),(20.0,99.0)));
        v.push(((25.0,1050.0),(25.0,99.0)));
        v.push(((30.0,1050.0),(30.0,99.0)));
        v.push(((35.0,1050.0),(35.0,99.0)));
        v.push(((40.0,1050.0),(40.0,99.0)));
        v.push(((45.0,1050.0),(45.0,99.0)));
        v.push(((50.0,1050.0),(50.0,99.0)));
        v.push(((55.0,1050.0),(55.0,99.0)));
        v.push(((60.0,1050.0),(60.0,99.0)));
        v
    };

    pub static ref ISO_MIXING_RATIO_PNTS: Vec<(TPCoords, TPCoords)> = {
        let mut v: Vec<(TPCoords, TPCoords)> = vec![];
        v.push(((-40.9,1050.0),(-52.0,300.0)));
        v.push(((-34.1,1050.0),(-46.0,300.0)));
        v.push(((-26.9,1050.0),(-39.6,300.0)));
        v.push(((-22.4,1050.0),(-35.7,300.0)));
        v.push(((-19.1,1050.0),(-32.8,300.0)));
        v.push(((-16.5,1050.0),(-30.5,300.0)));
        v.push(((-11.6,1050.0),(-26.1,300.0)));
        v.push(((-7.9,1050.0),(-23.0,300.0)));
        v.push(((-5.0,1050.0),(-20.4,300.0)));
        v.push(((-2.6,1050.0),(-18.3,300.0)));
        v.push(((1.3,1050.0),(-14.9,300.0)));
        v.push(((4.4,1050.0),(-12.2,300.0)));
        v.push(((7.0,1050.0),(-10.0,300.0)));
        v.push(((9.3,1050.0),(-8.0,300.0)));
        v.push(((11.2,1050.0),(-6.3,300.0)));
        v.push(((14.6,1050.0),(-3.4,300.0)));
        v.push(((17.4,1050.0),(-1.0,300.0)));
        v.push(((19.8,1050.0),(1.1,300.0)));
        v.push(((21.9,1050.0),(2.9,300.0)));
        v.push(((23.8,1050.0),(4.5,300.0)));
        v.push(((25.6,1050.0),(6.0,300.0)));
        v.push(((28.6,1050.0),(8.6,300.0)));
        v.push(((31.1,1050.0),(10.8,300.0)));
        v.push(((33.4,1050.0),(12.7,300.0)));
        v.push(((35.4,1050.0),(14.4,300.0)));
        v.push(((37.2,1050.0),(16.0,300.0)));
        v.push(((38.9,1050.0),(17.4,300.0)));
        v.push(((40.4,1050.0),(18.7,300.0)));
        v.push(((41.8,1050.0),(19.8,300.0)));
        v.push(((43.1,1050.0),(21.0,300.0)));
        v.push(((44.3,1050.0),(22.0,300.0)));
        v.push(((46.5,1050.0),(23.9,300.0)));
        v
    };

    pub static ref ISENTROP_PNTS: Vec<Vec<TPCoords>> = {
        let mut v: Vec<Vec<TPCoords>> = vec![];
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((-39.91807556,1050.0));
        vv.push((-41.51995850,1025.0));
        vv.push((-43.14999390,1000.0));
        vv.push((-44.80938721,975.0));
        vv.push((-46.49943542,950.0));
        vv.push((-48.22155762,925.0));
        vv.push((-49.97723389,900.0));
        vv.push((-51.76808167,875.0));
        vv.push((-53.59584045,850.0));
        vv.push((-55.46240234,825.0));
        vv.push((-57.36981201,800.0));
        vv.push((-59.32025146,775.0));
        vv.push((-61.31614685,750.0));
        vv.push((-63.36012268,725.0));
        vv.push((-65.45507812,700.0));
        vv.push((-67.60414124,675.0));
        vv.push((-69.81082153,650.0));
        vv.push((-72.07894897,625.0));
        vv.push((-74.41282654,600.0));
        vv.push((-76.81719971,575.0));
        vv.push((-79.29742432,550.0));
        vv.push((-81.85949707,525.0));
        vv.push((-84.51022339,500.0));
        vv.push((-87.25735474,475.0));
        vv.push((-90.10975647,450.0));
        vv.push((-93.07762146,425.0));
        vv.push((-96.17292786,400.0));
        vv.push((-99.40960693,375.0));
        vv.push((-102.80424500,350.0));
        vv.push((-106.37670898,325.0));
        vv.push((-110.15115356,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((-29.77755737,1050.0));
        vv.push((-31.44908142,1025.0));
        vv.push((-33.14999390,1000.0));
        vv.push((-34.88153076,975.0));
        vv.push((-36.64506531,950.0));
        vv.push((-38.44206238,925.0));
        vv.push((-40.27406311,900.0));
        vv.push((-42.14277649,875.0));
        vv.push((-44.05000305,850.0));
        vv.push((-45.99772644,825.0));
        vv.push((-47.98806763,800.0));
        vv.push((-50.02330017,775.0));
        vv.push((-52.10597229,750.0));
        vv.push((-54.23883057,725.0));
        vv.push((-56.42486572,700.0));
        vv.push((-58.66737366,675.0));
        vv.push((-60.97000122,650.0));
        vv.push((-63.33673096,625.0));
        vv.push((-65.77207947,600.0));
        vv.push((-68.28099060,575.0));
        vv.push((-70.86904907,550.0));
        vv.push((-73.54251099,525.0));
        vv.push((-76.30850220,500.0));
        vv.push((-79.17506409,475.0));
        vv.push((-82.15147400,450.0));
        vv.push((-85.24839783,425.0));
        vv.push((-88.47827148,400.0));
        vv.push((-91.85568237,375.0));
        vv.push((-95.39791870,350.0));
        vv.push((-99.12570190,325.0));
        vv.push((-103.06423950,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((-19.63703918,1050.0));
        vv.push((-21.37821960,1025.0));
        vv.push((-23.14999390,1000.0));
        vv.push((-24.95367432,975.0));
        vv.push((-26.79069519,950.0));
        vv.push((-28.66255188,925.0));
        vv.push((-30.57090759,900.0));
        vv.push((-32.51747131,875.0));
        vv.push((-34.50416565,850.0));
        vv.push((-36.53305054,825.0));
        vv.push((-38.60630798,800.0));
        vv.push((-40.72636414,775.0));
        vv.push((-42.89581299,750.0));
        vv.push((-45.11752319,725.0));
        vv.push((-47.39463806,700.0));
        vv.push((-49.73059082,675.0));
        vv.push((-52.12916565,650.0));
        vv.push((-54.59451294,625.0));
        vv.push((-57.13133240,600.0));
        vv.push((-59.74478149,575.0));
        vv.push((-62.44067383,550.0));
        vv.push((-65.22554016,525.0));
        vv.push((-68.10676575,500.0));
        vv.push((-71.09278870,475.0));
        vv.push((-74.19320679,450.0));
        vv.push((-77.41915894,425.0));
        vv.push((-80.78361511,400.0));
        vv.push((-84.30175781,375.0));
        vv.push((-87.99157715,350.0));
        vv.push((-91.87467957,325.0));
        vv.push((-95.97732544,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((-9.49652100,1050.0));
        vv.push((-11.30734253,1025.0));
        vv.push((-13.14999390,1000.0));
        vv.push((-15.02581787,975.0));
        vv.push((-16.93630981,950.0));
        vv.push((-18.88305664,925.0));
        vv.push((-20.86773682,900.0));
        vv.push((-22.89218140,875.0));
        vv.push((-24.95834351,850.0));
        vv.push((-27.06837463,825.0));
        vv.push((-29.22456360,800.0));
        vv.push((-31.42941284,775.0));
        vv.push((-33.68563843,750.0));
        vv.push((-35.99623108,725.0));
        vv.push((-38.36442566,700.0));
        vv.push((-40.79382324,675.0));
        vv.push((-43.28833008,650.0));
        vv.push((-45.85229492,625.0));
        vv.push((-48.49058533,600.0));
        vv.push((-51.20857239,575.0));
        vv.push((-54.01229858,550.0));
        vv.push((-56.90855408,525.0));
        vv.push((-59.90504456,500.0));
        vv.push((-63.01049805,475.0));
        vv.push((-66.23493958,450.0));
        vv.push((-69.58992004,425.0));
        vv.push((-73.08895874,400.0));
        vv.push((-76.74781799,375.0));
        vv.push((-80.58523560,350.0));
        vv.push((-84.62367249,325.0));
        vv.push((-88.89042664,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((0.64398193,1050.0));
        vv.push((-1.23648071,1025.0));
        vv.push((-3.14999390,1000.0));
        vv.push((-5.09796143,975.0));
        vv.push((-7.08193970,950.0));
        vv.push((-9.10357666,925.0));
        vv.push((-11.16458130,900.0));
        vv.push((-13.26687622,875.0));
        vv.push((-15.41250610,850.0));
        vv.push((-17.60369873,825.0));
        vv.push((-19.84281921,800.0));
        vv.push((-22.13247681,775.0));
        vv.push((-24.47546387,750.0));
        vv.push((-26.87492371,725.0));
        vv.push((-29.33421326,700.0));
        vv.push((-31.85704041,675.0));
        vv.push((-34.44749451,650.0));
        vv.push((-37.11007690,625.0));
        vv.push((-39.84983826,600.0));
        vv.push((-42.67236328,575.0));
        vv.push((-45.58392334,550.0));
        vv.push((-48.59158325,525.0));
        vv.push((-51.70330811,500.0));
        vv.push((-54.92820740,475.0));
        vv.push((-58.27665710,450.0));
        vv.push((-61.76069641,425.0));
        vv.push((-65.39430237,400.0));
        vv.push((-69.19389343,375.0));
        vv.push((-73.17890930,350.0));
        vv.push((-77.37266541,325.0));
        vv.push((-81.80352783,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((10.78451538,1050.0));
        vv.push((8.83441162,1025.0));
        vv.push((6.85000610,1000.0));
        vv.push((4.82989502,975.0));
        vv.push((2.77243042,950.0));
        vv.push((0.67593384,925.0));
        vv.push((-1.46142578,900.0));
        vv.push((-3.64157104,875.0));
        vv.push((-5.86666870,850.0));
        vv.push((-8.13900757,825.0));
        vv.push((-10.46105957,800.0));
        vv.push((-12.83551025,775.0));
        vv.push((-15.26528931,750.0));
        vv.push((-17.75363159,725.0));
        vv.push((-20.30400085,700.0));
        vv.push((-22.92027283,675.0));
        vv.push((-25.60665894,650.0));
        vv.push((-28.36785889,625.0));
        vv.push((-31.20909119,600.0));
        vv.push((-34.13616943,575.0));
        vv.push((-37.15556335,550.0));
        vv.push((-40.27459717,525.0));
        vv.push((-43.50158691,500.0));
        vv.push((-46.84591675,475.0));
        vv.push((-50.31838989,450.0));
        vv.push((-53.93145752,425.0));
        vv.push((-57.69964600,400.0));
        vv.push((-61.63996887,375.0));
        vv.push((-65.77256775,350.0));
        vv.push((-70.12164307,325.0));
        vv.push((-74.71661377,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((20.92501831,1050.0));
        vv.push((18.90527344,1025.0));
        vv.push((16.85000610,1000.0));
        vv.push((14.75772095,975.0));
        vv.push((12.62680054,950.0));
        vv.push((10.45544434,925.0));
        vv.push((8.24176025,900.0));
        vv.push((5.98373413,875.0));
        vv.push((3.67916870,850.0));
        vv.push((1.32565308,825.0));
        vv.push((-1.07931519,800.0));
        vv.push((-3.53857422,775.0));
        vv.push((-6.05514526,750.0));
        vv.push((-8.63232422,725.0));
        vv.push((-11.27380371,700.0));
        vv.push((-13.98348999,675.0));
        vv.push((-16.76583862,650.0));
        vv.push((-19.62564087,625.0));
        vv.push((-22.56834412,600.0));
        vv.push((-25.59996033,575.0));
        vv.push((-28.72718811,550.0));
        vv.push((-31.95762634,525.0));
        vv.push((-35.29985046,500.0));
        vv.push((-38.76362610,475.0));
        vv.push((-42.36012268,450.0));
        vv.push((-46.10221863,425.0));
        vv.push((-50.00498962,400.0));
        vv.push((-54.08602905,375.0));
        vv.push((-58.36622620,350.0));
        vv.push((-62.87063599,325.0));
        vv.push((-67.62969971,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((31.06555176,1050.0));
        vv.push((28.97613525,1025.0));
        vv.push((26.85000610,1000.0));
        vv.push((24.68557739,975.0));
        vv.push((22.48117065,950.0));
        vv.push((20.23492432,925.0));
        vv.push((17.94491577,900.0));
        vv.push((15.60903931,875.0));
        vv.push((13.22500610,850.0));
        vv.push((10.79034424,825.0));
        vv.push((8.30242920,800.0));
        vv.push((5.75836182,775.0));
        vv.push((3.15502930,750.0));
        vv.push((0.48898315,725.0));
        vv.push((-2.24356079,700.0));
        vv.push((-5.04672241,675.0));
        vv.push((-7.92498779,650.0));
        vv.push((-10.88342285,625.0));
        vv.push((-13.92761230,600.0));
        vv.push((-17.06375122,575.0));
        vv.push((-20.29881287,550.0));
        vv.push((-23.64064026,525.0));
        vv.push((-27.09812927,500.0));
        vv.push((-30.68133545,475.0));
        vv.push((-34.40185547,450.0));
        vv.push((-38.27299500,425.0));
        vv.push((-42.31034851,400.0));
        vv.push((-46.53210449,375.0));
        vv.push((-50.95989990,350.0));
        vv.push((-55.61962891,325.0));
        vv.push((-60.54280090,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((41.20605469,1050.0));
        vv.push((39.04699707,1025.0));
        vv.push((36.85000610,1000.0));
        vv.push((34.61343384,975.0));
        vv.push((32.33554077,950.0));
        vv.push((30.01443481,925.0));
        vv.push((27.64807129,900.0));
        vv.push((25.23434448,875.0));
        vv.push((22.77081299,850.0));
        vv.push((20.25500488,825.0));
        vv.push((17.68417358,800.0));
        vv.push((15.05529785,775.0));
        vv.push((12.36520386,750.0));
        vv.push((9.61026001,725.0));
        vv.push((6.78665161,700.0));
        vv.push((3.89004517,675.0));
        vv.push((0.91583252,650.0));
        vv.push((-2.14120483,625.0));
        vv.push((-5.28686523,600.0));
        vv.push((-8.52752686,575.0));
        vv.push((-11.87045288,550.0));
        vv.push((-15.32366943,525.0));
        vv.push((-18.89639282,500.0));
        vv.push((-22.59904480,475.0));
        vv.push((-26.44357300,450.0));
        vv.push((-30.44375610,425.0));
        vv.push((-34.61569214,400.0));
        vv.push((-38.97817993,375.0));
        vv.push((-43.55355835,350.0));
        vv.push((-48.36860657,325.0));
        vv.push((-53.45590210,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((51.34658813,1050.0));
        vv.push((49.11788940,1025.0));
        vv.push((46.85000610,1000.0));
        vv.push((44.54129028,975.0));
        vv.push((42.18991089,950.0));
        vv.push((39.79391479,925.0));
        vv.push((37.35122681,900.0));
        vv.push((34.85961914,875.0));
        vv.push((32.31665039,850.0));
        vv.push((29.71969604,825.0));
        vv.push((27.06591797,800.0));
        vv.push((24.35226440,775.0));
        vv.push((21.57537842,750.0));
        vv.push((18.73156738,725.0));
        vv.push((15.81686401,700.0));
        vv.push((12.82684326,675.0));
        vv.push((9.75668335,650.0));
        vv.push((6.60101318,625.0));
        vv.push((3.35388184,600.0));
        vv.push((0.00866699,575.0));
        vv.push((-3.44207764,550.0));
        vv.push((-7.00668335,525.0));
        vv.push((-10.69467163,500.0));
        vv.push((-14.51675415,475.0));
        vv.push((-18.48530579,450.0));
        vv.push((-22.61453247,425.0));
        vv.push((-26.92103577,400.0));
        vv.push((-31.42424011,375.0));
        vv.push((-36.14721680,350.0));
        vv.push((-41.11759949,325.0));
        vv.push((-46.36898804,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((61.48709106,1050.0));
        vv.push((59.18875122,1025.0));
        vv.push((56.85000610,1000.0));
        vv.push((54.46914673,975.0));
        vv.push((52.04428101,950.0));
        vv.push((49.57342529,925.0));
        vv.push((47.05441284,900.0));
        vv.push((44.48492432,875.0));
        vv.push((41.86248779,850.0));
        vv.push((39.18435669,825.0));
        vv.push((36.44766235,800.0));
        vv.push((33.64920044,775.0));
        vv.push((30.78552246,750.0));
        vv.push((27.85287476,725.0));
        vv.push((24.84707642,700.0));
        vv.push((21.76361084,675.0));
        vv.push((18.59750366,650.0));
        vv.push((15.34323120,625.0));
        vv.push((11.99462891,600.0));
        vv.push((8.54489136,575.0));
        vv.push((4.98629761,550.0));
        vv.push((1.31027222,525.0));
        vv.push((-2.49291992,500.0));
        vv.push((-6.43447876,475.0));
        vv.push((-10.52703857,450.0));
        vv.push((-14.78527832,425.0));
        vv.push((-19.22637939,400.0));
        vv.push((-23.87031555,375.0));
        vv.push((-28.74089050,350.0));
        vv.push((-33.86659241,325.0));
        vv.push((-39.28207397,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((71.62762451,1050.0));
        vv.push((69.25961304,1025.0));
        vv.push((66.85000610,1000.0));
        vv.push((64.39700317,975.0));
        vv.push((61.89865112,950.0));
        vv.push((59.35290527,925.0));
        vv.push((56.75756836,900.0));
        vv.push((54.11022949,875.0));
        vv.push((51.40832520,850.0));
        vv.push((48.64904785,825.0));
        vv.push((45.82940674,800.0));
        vv.push((42.94613647,775.0));
        vv.push((39.99569702,750.0));
        vv.push((36.97415161,725.0));
        vv.push((33.87728882,700.0));
        vv.push((30.70037842,675.0));
        vv.push((27.43832397,650.0));
        vv.push((24.08544922,625.0));
        vv.push((20.63537598,600.0));
        vv.push((17.08108521,575.0));
        vv.push((13.41467285,550.0));
        vv.push((9.62725830,525.0));
        vv.push((5.70880127,500.0));
        vv.push((1.64782715,475.0));
        vv.push((-2.56875610,450.0));
        vv.push((-6.95605469,425.0));
        vv.push((-11.53170776,400.0));
        vv.push((-16.31637573,375.0));
        vv.push((-21.33454895,350.0));
        vv.push((-26.61557007,325.0));
        vv.push((-32.19517517,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((81.76812744,1050.0));
        vv.push((79.33050537,1025.0));
        vv.push((76.85000610,1000.0));
        vv.push((74.32485962,975.0));
        vv.push((71.75302124,950.0));
        vv.push((69.13241577,925.0));
        vv.push((66.46072388,900.0));
        vv.push((63.73553467,875.0));
        vv.push((60.95416260,850.0));
        vv.push((58.11370850,825.0));
        vv.push((55.21115112,800.0));
        vv.push((52.24310303,775.0));
        vv.push((49.20587158,750.0));
        vv.push((46.09545898,725.0));
        vv.push((42.90750122,700.0));
        vv.push((39.63717651,675.0));
        vv.push((36.27917480,650.0));
        vv.push((32.82766724,625.0));
        vv.push((29.27612305,600.0));
        vv.push((25.61730957,575.0));
        vv.push((21.84304810,550.0));
        vv.push((17.94424438,525.0));
        vv.push((13.91052246,500.0));
        vv.push((9.73010254,475.0));
        vv.push((5.38949585,450.0));
        vv.push((0.87316895,425.0));
        vv.push((-3.83706665,400.0));
        vv.push((-8.76245117,375.0));
        vv.push((-13.92822266,350.0));
        vv.push((-19.36456299,325.0));
        vv.push((-25.10827637,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((91.90866089,1050.0));
        vv.push((89.40136719,1025.0));
        vv.push((86.85000610,1000.0));
        vv.push((84.25271606,975.0));
        vv.push((81.60739136,950.0));
        vv.push((78.91192627,925.0));
        vv.push((76.16387939,900.0));
        vv.push((73.36083984,875.0));
        vv.push((70.50000000,850.0));
        vv.push((67.57839966,825.0));
        vv.push((64.59289551,800.0));
        vv.push((61.54003906,775.0));
        vv.push((58.41604614,750.0));
        vv.push((55.21676636,725.0));
        vv.push((51.93771362,700.0));
        vv.push((48.57394409,675.0));
        vv.push((45.11999512,650.0));
        vv.push((41.56988525,625.0));
        vv.push((37.91687012,600.0));
        vv.push((34.15350342,575.0));
        vv.push((30.27142334,550.0));
        vv.push((26.26123047,525.0));
        vv.push((22.11224365,500.0));
        vv.push((17.81237793,475.0));
        vv.push((13.34777832,450.0));
        vv.push((8.70242310,425.0));
        vv.push((3.85757446,400.0));
        vv.push((-1.20852661,375.0));
        vv.push((-6.52188110,350.0));
        vv.push((-12.11355591,325.0));
        vv.push((-18.02136230,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((102.04916382,1050.0));
        vv.push((99.47222900,1025.0));
        vv.push((96.85000610,1000.0));
        vv.push((94.18057251,975.0));
        vv.push((91.46176147,950.0));
        vv.push((88.69140625,925.0));
        vv.push((85.86706543,900.0));
        vv.push((82.98614502,875.0));
        vv.push((80.04583740,850.0));
        vv.push((77.04309082,825.0));
        vv.push((73.97463989,800.0));
        vv.push((70.83697510,775.0));
        vv.push((67.62619019,750.0));
        vv.push((64.33807373,725.0));
        vv.push((60.96792603,700.0));
        vv.push((57.51071167,675.0));
        vv.push((53.96084595,650.0));
        vv.push((50.31210327,625.0));
        vv.push((46.55761719,600.0));
        vv.push((42.68969727,575.0));
        vv.push((38.69979858,550.0));
        vv.push((34.57818604,525.0));
        vv.push((30.31399536,500.0));
        vv.push((25.89468384,475.0));
        vv.push((21.30606079,450.0));
        vv.push((16.53164673,425.0));
        vv.push((11.55224609,400.0));
        vv.push((6.34539795,375.0));
        vv.push((0.88446045,350.0));
        vv.push((-4.86254883,325.0));
        vv.push((-10.93444824,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((112.18969727,1050.0));
        vv.push((109.54312134,1025.0));
        vv.push((106.85000610,1000.0));
        vv.push((104.10839844,975.0));
        vv.push((101.31616211,950.0));
        vv.push((98.47091675,925.0));
        vv.push((95.57022095,900.0));
        vv.push((92.61145020,875.0));
        vv.push((89.59164429,850.0));
        vv.push((86.50775146,825.0));
        vv.push((83.35641479,800.0));
        vv.push((80.13394165,775.0));
        vv.push((76.83636475,750.0));
        vv.push((73.45935059,725.0));
        vv.push((69.99813843,700.0));
        vv.push((66.44747925,675.0));
        vv.push((62.80166626,650.0));
        vv.push((59.05432129,625.0));
        vv.push((55.19836426,600.0));
        vv.push((51.22592163,575.0));
        vv.push((47.12817383,550.0));
        vv.push((42.89517212,525.0));
        vv.push((38.51571655,500.0));
        vv.push((33.97695923,475.0));
        vv.push((29.26431274,450.0));
        vv.push((24.36087036,425.0));
        vv.push((19.24688721,400.0));
        vv.push((13.89932251,375.0));
        vv.push((8.29080200,350.0));
        vv.push((2.38848877,325.0));
        vv.push((-3.84756470,300.0));
        v.push(vv);
        let mut vv: Vec<TPCoords> = vec![];
        vv.push((122.33020020,1050.0));
        vv.push((119.61398315,1025.0));
        vv.push((116.85000610,1000.0));
        vv.push((114.03625488,975.0));
        vv.push((111.17053223,950.0));
        vv.push((108.25039673,925.0));
        vv.push((105.27337646,900.0));
        vv.push((102.23672485,875.0));
        vv.push((99.13748169,850.0));
        vv.push((95.97244263,825.0));
        vv.push((92.73815918,800.0));
        vv.push((89.43087769,775.0));
        vv.push((86.04653931,750.0));
        vv.push((82.58065796,725.0));
        vv.push((79.02835083,700.0));
        vv.push((75.38427734,675.0));
        vv.push((71.64251709,650.0));
        vv.push((67.79656982,625.0));
        vv.push((63.83911133,600.0));
        vv.push((59.76211548,575.0));
        vv.push((55.55654907,550.0));
        vv.push((51.21215820,525.0));
        vv.push((46.71743774,500.0));
        vv.push((42.05926514,475.0));
        vv.push((37.22259521,450.0));
        vv.push((32.19009399,425.0));
        vv.push((26.94155884,400.0));
        vv.push((21.45327759,375.0));
        vv.push((15.69714355,350.0));
        vv.push((9.63949585,325.0));
        vv.push((3.23934937,300.0));
        v.push(vv);
        v
    };

}
