package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"sync"
	"sync/atomic"
)

func main() {
	var large uint64
	var wg sync.WaitGroup

	var threads = make(chan int, 1000)
	defer close(threads)

	dir := "/data/flickr-korea/faces"
	files, err := ioutil.ReadDir(dir)
	if err != nil {
		log.Fatal(err)
	}

	for _, f := range files {
		wg.Add(1)
		threads <- 1
		go func(f string) {
			defer wg.Done()
			defer func() { <-threads }()
			file, err := os.Open(dir + f)
			if err != nil {
				return
			}
			stat, err := file.Stat()
			if err != nil {
				return
			}
			if stat.Size() > 10*1024*1024 {
				atomic.AddUint64(&large, 1)
			}
		}(f.Name())
	}

	wg.Wait()
	fmt.Println("filesize>10M : ", large)
}
