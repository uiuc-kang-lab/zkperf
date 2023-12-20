package gnarkml

import "github.com/consensys/gnark/frontend"

type MnistImage struct {
	Image [][]int
}

type Layer struct {
	Layer_type string
	Inp_idxes  []int
	Inp_shapes [][]int
	Out_idxes  []int
	Out_shapes [][]int
	Params     []int
	Mask       []string
}

type Tensor struct {
	Idx   int
	Shape []int
	Data  []int
}

type Cfg struct {
	Global_sf     int
	K             int
	Num_cols      int
	Num_random    int
	Inp_idxes     []int
	Out_idxes     []int
	Layers        []Layer
	Tensors       []Tensor
	Use_selector  bool
	Commit_before []int
	Commit_after  []int
}

func get_tensor(tensor_idx int, tensors []Tensor) Tensor {
	var required_tensor Tensor
	for i := range tensors {
		if tensors[i].Idx == tensor_idx {
			required_tensor = tensors[i]
		}
	}
	return required_tensor
}

func make_conv_kernel(api frontend.API, tensor Tensor) [][][][]frontend.Variable {
	shape := tensor.Shape
	data := tensor.Data
	transformed_kernel := make([][][][]frontend.Variable, shape[0])
	index := 0
	for i := 0; i < shape[0]; i++ {
		transformed_kernel[i] = make([][][]frontend.Variable, shape[1])
		for j := 0; j < shape[1]; j++ {
			transformed_kernel[i][j] = make([][]frontend.Variable, shape[2])
			for k := 0; k < shape[2]; k++ {
				transformed_kernel[i][j][k] = make([]frontend.Variable, shape[3])
				for l := 0; l < shape[3]; l++ {
					transformed_kernel[i][j][k][l] = frontend.Variable(data[index])
					index += 1
				}
			}
		}
	}
	return transformed_kernel
}

func make_conv_bias(api frontend.API, tensor Tensor) []frontend.Variable {
	shape := tensor.Shape
	data := tensor.Data
	transformed_bias := make([]frontend.Variable, shape[0])
	for i := 0; i < shape[0]; i++ {
		transformed_bias[i] = frontend.Variable(data[i])
	}
	return transformed_bias
}

func make_weight(api frontend.API, tensor Tensor) [][]frontend.Variable {
	shape := tensor.Shape
	data := tensor.Data
	transformed_kernel := make([][]frontend.Variable, shape[0])
	index := 0
	for i := 0; i < shape[0]; i++ {
		transformed_kernel[i] = make([]frontend.Variable, shape[1])
		for j := 0; j < shape[1]; j++ {
			transformed_kernel[i][j] = frontend.Variable(data[index])
			index += 1
		}
	}

	return transformed_kernel
}

func make_bias(api frontend.API, tensor Tensor) []frontend.Variable {
	shape := tensor.Shape
	data := tensor.Data
	transformed_bias := make([]frontend.Variable, shape[0])
	for i := 0; i < shape[0]; i++ {
		transformed_bias[i] = frontend.Variable(data[i])
	}
	return transformed_bias
}

func check_err(err error) {
	if err != nil {
		panic(err)
	}
}

func get_gather_idx() []int {
	gather_idx := []int{27,
		54,
		55,
		81,
		82,
		83,
		108,
		109,
		110,
		111,
		135,
		136,
		137,
		138,
		139,
		162,
		163,
		164,
		165,
		166,
		167,
		189,
		190,
		191,
		192,
		193,
		194,
		195,
		216,
		217,
		218,
		219,
		220,
		221,
		222,
		223,
		243,
		244,
		245,
		246,
		247,
		248,
		249,
		250,
		251,
		270,
		271,
		272,
		273,
		274,
		275,
		276,
		277,
		278,
		279,
		297,
		298,
		299,
		300,
		301,
		302,
		303,
		304,
		305,
		306,
		307,
		324,
		325,
		326,
		327,
		328,
		329,
		330,
		331,
		332,
		333,
		334,
		335,
		351,
		352,
		353,
		354,
		355,
		356,
		357,
		358,
		359,
		360,
		361,
		362,
		363,
		378,
		379,
		380,
		381,
		382,
		383,
		384,
		385,
		386,
		387,
		388,
		389,
		390,
		391,
		405,
		406,
		407,
		408,
		409,
		410,
		411,
		412,
		413,
		414,
		415,
		416,
		417,
		418,
		419,
		432,
		433,
		434,
		435,
		436,
		437,
		438,
		439,
		440,
		441,
		442,
		443,
		444,
		445,
		446,
		447,
		459,
		460,
		461,
		462,
		463,
		464,
		465,
		466,
		467,
		468,
		469,
		470,
		471,
		472,
		473,
		474,
		475,
		486,
		487,
		488,
		489,
		490,
		491,
		492,
		493,
		494,
		495,
		496,
		497,
		498,
		499,
		500,
		501,
		502,
		503,
		513,
		514,
		515,
		516,
		517,
		518,
		519,
		520,
		521,
		522,
		523,
		524,
		525,
		526,
		527,
		528,
		529,
		530,
		531,
		540,
		541,
		542,
		543,
		544,
		545,
		546,
		547,
		548,
		549,
		550,
		551,
		552,
		553,
		554,
		555,
		556,
		557,
		558,
		559,
		567,
		568,
		569,
		570,
		571,
		572,
		573,
		574,
		575,
		576,
		577,
		578,
		579,
		580,
		581,
		582,
		583,
		584,
		585,
		586,
		587,
		594,
		595,
		596,
		597,
		598,
		599,
		600,
		601,
		602,
		603,
		604,
		605,
		606,
		607,
		608,
		609,
		610,
		611,
		612,
		613,
		614,
		615,
		621,
		622,
		623,
		624,
		625,
		626,
		627,
		628,
		629,
		630,
		631,
		632,
		633,
		634,
		635,
		636,
		637,
		638,
		639,
		640,
		641,
		642,
		643,
		648,
		649,
		650,
		651,
		652,
		653,
		654,
		655,
		656,
		657,
		658,
		659,
		660,
		661,
		662,
		663,
		664,
		665,
		666,
		667,
		668,
		669,
		670,
		671,
		675,
		676,
		677,
		678,
		679,
		680,
		681,
		682,
		683,
		684,
		685,
		686,
		687,
		688,
		689,
		690,
		691,
		692,
		693,
		694,
		695,
		696,
		697,
		698,
		699,
		702,
		703,
		704,
		705,
		706,
		707,
		708,
		709,
		710,
		711,
		712,
		713,
		714,
		715,
		716,
		717,
		718,
		719,
		720,
		721,
		722,
		723,
		724,
		725,
		726,
		727}
	return gather_idx
}
