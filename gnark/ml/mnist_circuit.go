package gnarkml

import (
	"encoding/json"
	"fmt"
	"math"
	"os"

	"github.com/consensys/gnark/frontend"
)

/*
	Meaning of params in json
	For Conv2D: [0, padding(0 for same, 1 for valid), Activation, Stride_height, Stride_width]
	For Depthwise [0, padding(0 for same, 1 for valid), Activation, Stride_height, Stride_width]
	For AvgPool2D: [Filter_height, Filter_width, Stride_height, Stride_width]
	Add for adding bias,
*/

func MNIST_ZK(api frontend.API, Image [][][]frontend.Variable, reluoff bool, numlayers int) [][][]frontend.Variable {
	content, err := os.ReadFile("ml/converted_mnist_model_sf512.json")
	check_err(err)
	var modelConfig Cfg
	err = json.Unmarshal(content, &modelConfig)
	check_err(err)
	N := int(math.Log2(float64(modelConfig.Global_sf)))
	fmt.Println("N: ", N)
	Global_sf := frontend.Variable(modelConfig.Global_sf)
	var output [][][]frontend.Variable
	output = Image
	fmt.Println("No of layers: ", len(modelConfig.Layers))
	for i := 0; i < len(modelConfig.Layers); i++ {
		layer := modelConfig.Layers[i]
		if layer.Layer_type == "Conv2D" && layer.Params[0] == 0 {
			kernel := get_tensor(layer.Inp_idxes[1], modelConfig.Tensors)
			bias := get_tensor(layer.Inp_idxes[2], modelConfig.Tensors)
			transformed_kernel := make_conv_kernel(api, kernel)
			tranformed_bias := make_conv_bias(api, bias)
			if layer.Params[2] == 0 {
				reluoff = true
			}
			fmt.Println("Kernel Shape:", len(transformed_kernel), len(transformed_kernel[0]), len(transformed_kernel[0][0]), len(transformed_kernel[0][0][0]))
			fmt.Println("Bias Shape:", len(tranformed_bias))
			output = Conv2D(api, output, transformed_kernel, tranformed_bias, layer.Params[3], layer.Params[1], reluoff, Global_sf, N)
			fmt.Println(layer)
		} else if layer.Layer_type == "AveragePool2D" {
			output = AvgPool2D(api, output, layer.Params, Global_sf, N)
			fmt.Println(layer)
		} else if layer.Layer_type == "Mul" {
			scale := get_tensor(layer.Inp_idxes[1], modelConfig.Tensors)
			transformed_scale := make_conv_bias(api, scale)
			output = Mul(api, output, transformed_scale, Global_sf, N)
			fmt.Println(layer)
		} else if layer.Layer_type == "Add" {
			bias := get_tensor(layer.Inp_idxes[1], modelConfig.Tensors)
			transformed_bias := make_conv_bias(api, bias)
			output = Add(api, output, transformed_bias, Global_sf, N)
			fmt.Println(layer)
		} else {
			fmt.Println("Layer", layer.Layer_type, " not supported")
		}
	}
	return output
}

type MNISTCircuit struct {
	Image [][][]frontend.Variable `gnark:",public"`
}

func (circuit *MNISTCircuit) Define(api frontend.API) error {
	op := MNIST_ZK(api, circuit.Image, false, 16)
	for i := 0; i < len(op); i++ {
		for j := 0; j < len(op[0]); j++ {
			for k := 0; k < len(op[0][0]); k++ {
				api.Println(op[i][j][k])
			}
		}
	}
	return nil
}
