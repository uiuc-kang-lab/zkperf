package main

import (
	"fmt"
	"runtime"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

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
	msm_time := time.Since(start)
	fmt.Println("Time taken by G1Aff MSM:", msm_time)

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
	msm_time = time.Since(start)
	fmt.Println("Time taken by G2Aff MSM:", msm_time)
}
