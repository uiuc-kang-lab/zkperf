package main

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"strconv"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

type MSMResult struct {
	Framework string
	Size      int
	G1time    int64
	G2time    int64
}

func DummyMSMAffine(n int) {
	numcpu := runtime.NumCPU()
	scalars := make([]fr.Element, n)
	fmt.Println("MSM Size:", n)
	fmt.Println("No of CPU:", numcpu)
	// G1Affine
	g1_elements := make([]bn254.G1Affine, n)
	_, _, g1, _ := bn254.Generators()
	var result_g1 bn254.G1Affine
	for i := 0; i < n; i++ {
		scalars[i].SetUint64(uint64(i))
		g1_elements[i] = g1
	}
	start := time.Now()
	result_g1.MultiExp(g1_elements, scalars, ecc.MultiExpConfig{NbTasks: numcpu / 2})
	g1_msm_time := time.Since(start)
	fmt.Println("Time taken by G1Aff MSM:", g1_msm_time)

	//G2Affine
	g2_elements := make([]bn254.G2Affine, n)
	_, _, _, g2 := bn254.Generators()
	var result_g2 bn254.G2Affine
	for i := 0; i < n; i++ {
		scalars[i].SetRandom()
		g2_elements[i] = g2
	}
	start = time.Now()
	result_g2.MultiExp(g2_elements, scalars, ecc.MultiExpConfig{NbTasks: numcpu / 2})
	g2_msm_time := time.Since(start)
	fmt.Println("Time taken by G2Aff MSM:", g2_msm_time)

	result := MSMResult{
		Framework: "gnark",
		Size:      n,
		G1time:    g1_msm_time.Nanoseconds(),
		G2time:    g2_msm_time.Nanoseconds(),
	}

	jsonfile, _ := json.MarshalIndent(result, "", " ")
	_ = os.WriteFile(strconv.Itoa(n)+"msm_timing.json", jsonfile, 0644)
}
