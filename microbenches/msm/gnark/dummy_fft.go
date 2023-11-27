package main

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"strconv"
	"time"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/fft"
)

type FFTResult struct {
	Framework string
	Size      int
	FFTtime   int64
	IFFTtime  int64
}

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
	fft_time_taken := time.Since(fft_time).Nanoseconds()
	fmt.Println("FFT Time:", time.Since(fft_time))

	ifft_time := time.Now()
	domain.FFTInverse(samples, fft.DIF)
	ifft_time_taken := time.Since(ifft_time).Nanoseconds()
	fmt.Println("IFFT Time:", time.Since(ifft_time))
	result := FFTResult{
		Framework: "gnark",
		Size:      n,
		FFTtime:   fft_time_taken,
		IFFTtime:  ifft_time_taken,
	}

	jsonfile, _ := json.MarshalIndent(result, "", " ")
	_ = os.WriteFile(strconv.Itoa(n)+"fft_timing.json", jsonfile, 0644)
}
