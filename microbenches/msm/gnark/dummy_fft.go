package main

import (
	"fmt"
	"runtime"
	"time"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/fft"
)

func DummyFFT(n int) {
	domain := fft.NewDomain(uint64(n))
	fmt.Println("Sample Size:", n)
	fmt.Println("No of CPU:", runtime.NumCPU())
	samples := make([]fr.Element, n)
	for i := 0; i < n; i++ {
		samples[i].SetUint64(uint64(i))
	}

	fft_time := time.Now()
	domain.FFT(samples, fft.DIT)
	fmt.Println("FFT Time:", time.Since(fft_time))

	ifft_time := time.Now()
	domain.FFTInverse(samples, fft.DIF)
	fmt.Println("IFFT Time:", time.Since(ifft_time))
}
