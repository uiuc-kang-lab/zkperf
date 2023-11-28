package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

type ArithmeticResult struct {
	Framework string
	Size      int
	Multime   int64
	Addtime   int64
}

func DummyArithmetic(n int) {
	fmt.Println("Sample Size:", n)
	samples := make([]fr.Element, n)
	scaled_samples := make([]fr.Element, n)
	var scale fr.Element
	var sum fr.Element
	sum.SetZero()
	scale.SetUint64(uint64(n / 2))
	start := time.Now()
	for i := 1; i <= n; i++ {
		samples[i].SetUint64(uint64(i))
		scaled_samples[i].Mul(&samples[i], &scale)
	}
	mul_time := time.Since(start)

	start = time.Now()
	for i := 0; i < n; i++ {
		sum.Add(&sum, &scaled_samples[i])
	}
	add_time := time.Since(start)

	fmt.Println("Sum: ", sum)

	result := ArithmeticResult{
		Framework: "gnark",
		Size:      n,
		Multime:   mul_time.Nanoseconds(),
		Addtime:   add_time.Nanoseconds(),
	}

	jsonfile, _ := json.MarshalIndent(result, "", " ")
	_ = os.WriteFile(strconv.Itoa(n)+"addmul_timing.json", jsonfile, 0644)
}
