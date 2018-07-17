package main

import (
	"fmt"
	"image"
	"io/ioutil"
	"log"
	"os"
	"sync"
	"sync/atomic"

	"gopkg.in/cheggaaa/pb.v1"

	_ "image/jpeg"
	_ "image/png"
)

func main() {
	var small uint64
	var wg sync.WaitGroup

	var threads = make(chan int, 1000)
	defer close(threads)

	dir := "/data/flickr-korea/faces/"
	files, err := ioutil.ReadDir(dir)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("total ", len(files))

	bar := pb.StartNew(len(files))

	for _, f := range files {
		wg.Add(1)
		threads <- 1
		go func(f string) {
			defer wg.Done()
			defer func() { <-threads }()
			file, err := os.Open(dir + f)
			defer file.Close()
			if err != nil {
				return
			}

			img, _, err := image.DecodeConfig(file)
			if img.Width < 512 && img.Height < 512 {
				atomic.AddUint64(&small, 1)
			}
			bar.Increment()
		}(f.Name())
	}
	bar.Finish()

	wg.Wait()
	fmt.Println("image short<512 : ", small)
}
