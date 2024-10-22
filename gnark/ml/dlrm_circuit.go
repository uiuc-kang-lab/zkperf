// oggn
package gnarkml

import (
	"encoding/json"
	"fmt"
	"log"
	"math"
	"os"

	"github.com/consensys/gnark/frontend"
)

func DLRM_ZK(api frontend.API, DenseLayers [][][]frontend.Variable, SparseLayer [][]frontend.Variable) [][]frontend.Variable {
	content, err := os.ReadFile("ml/converted_dlrm_checked_512.json")
	if err != nil {
		log.Fatal("Error when opening file: ", err)
	}
	var modelConfig Cfg
	output := SparseLayer
	err = json.Unmarshal(content, &modelConfig)
	if err != nil {
		log.Fatal("Error during Unmarshal(): ", err)
	}
	N := int(math.Log2(float64(modelConfig.Global_sf)))
	Global_sf := frontend.Variable(modelConfig.Global_sf)
	for i := 0; i < 3; i++ {
		layer := modelConfig.Layers[i]
		if layer.Layer_type == "FullyConnected" {
			weight := get_tensor(layer.Inp_idxes[1], modelConfig.Tensors)
			transformed_weight := make_weight(api, weight)
			bias := get_tensor(layer.Inp_idxes[2], modelConfig.Tensors)
			transformed_bias := make_bias(api, bias)
			output = FullyConnected(api, output, transformed_weight, transformed_bias, layer.Params, Global_sf, N, false)
		} else {
			panic("Error, layer should be FullyConnected")
		}
	}
	output_copy_for_later := output
	for j := 0; j < len(DenseLayers); j++ {
		output = Concatenate(api, output, DenseLayers[j])
	}

	reshaped_output := Reshape1Dto2D(api, output, []int{27, 64})
	output = FullyConnected(api, reshaped_output, reshaped_output, []frontend.Variable{0}, []int{0}, Global_sf, N, true)
	reshaped_output = Reshape2Dto1D(api, output, []int{729, 1})
	gather_idx := get_gather_idx()
	gather_output := make([][]frontend.Variable, 351)
	for i := 0; i < 351; i++ {
		gather_output[i] = make([]frontend.Variable, 1)
		gather_output[i][0] = reshaped_output[gather_idx[i]][0]
	}
	output = Transpose(api, gather_output)
	output = Concatenate(api, output_copy_for_later, output)
	for i := 10; i < 14; i++ {
		layer := modelConfig.Layers[i]
		if layer.Layer_type == "FullyConnected" {
			weight := get_tensor(layer.Inp_idxes[1], modelConfig.Tensors)
			transformed_weight := make_weight(api, weight)
			bias := get_tensor(layer.Inp_idxes[2], modelConfig.Tensors)
			transformed_bias := make_bias(api, bias)
			output = FullyConnected(api, output, transformed_weight, transformed_bias, layer.Params, Global_sf, N, false)
			fmt.Println(layer)
		} else {
			panic("Error, layer should be FullyConnected")
		}
	}
	return output
}

type DLRMCircuit struct {
	DenseLayers [][][]frontend.Variable `gnark:",public"`
	SparseLayer [][]frontend.Variable   `gnark:",public"`
}

func (circuit *DLRMCircuit) Define(api frontend.API) error {
	op := DLRM_ZK(api, circuit.DenseLayers, circuit.SparseLayer)
	for i := 0; i < len(op); i++ {
		for j := 0; j < len(op[0]); j++ {
			api.Println("Output: ", op[i][j])
		}
	}

	return nil
}
