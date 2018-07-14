package main

import (
	"errors"
	"fmt"
	"image"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"

	"github.com/karrick/godirwalk"
	"golang.org/x/sync/errgroup"

	"image/color"
	_ "image/jpeg"
	_ "image/png"
)

func printUsageAndPanic() {
	fmt.Println("Image black\n")
	fmt.Println("Usage: image-black [mode] [attr] (arguments) [src dir] (dst dir)")
	os.Exit(1)
}

var filter map[string]interface{}
var readHeader bool
var readContent bool

func cmp(s1 int, s2 int, op string) bool {
	switch op {
	case "==":
		return s1 == s2
	case ">":
		return s1 > s2
	case ">=":
		return s1 >= s2
	case "<":
		return s1 < s2
	case "<=":
		return s1 <= s2
	}
	return false
}

func Min(x, y int) int {
	if x < y {
		return x
	}
	return y
}

func Max(x, y int) int {
	if x > y {
		return x
	}
	return y
}

func check(path string) (bool, error) {
	if filter["format"] != false {
		ext := strings.ToLower(filepath.Ext(path))
		if ext != filter["format"] {
			return false, nil
		}
	}

	if filter["filesize"] != false {
		fi, e := os.Stat("/path/to/file")
		if e != nil {
			return false, e
		}
		s1 := int(fi.Size())
		s2 := filter["filesize"].(int)
		if !cmp(s1, s2, filter["filesize_op"].(string)) {
			return false, nil
		}
	}

	if readHeader {
		f, err := os.Open(path)
		if err != nil {
			return false, errors.New("fail to open image:  " + path)
		}
		defer f.Close()

		width, height := 0, 0
		channel := color.GrayModel
		valid := true

		if !readContent {
			img, _, err := image.DecodeConfig(f)
			valid := (err == nil)
			width, height = img.Width, img.Height
			channel = img.ColorModel
		} else {
			img, _, err := image.Decode(f)
			valid := (err == nil)
			b := img.Bounds()
			width, height = b.Max.X, b.Max.Y
			channel = img.ColorModel()
		}

		if filter["valid"] != false {
			if valid != (filter["valid"] == "valid") {
				return false, nil
			}
		}
		if filter["width"] != false && !cmp(width, filter["width"].(int), filter["width_op"].(string)) {
			return false, nil
		}
		if filter["height"] != false && !cmp(height, filter["height"].(int), filter["height_op"].(string)) {
			return false, nil
		}
		if filter["long"] != false {
			long := Max(width, height)
			if !cmp(long, filter["long"].(int), filter["long_op"].(string)) {
				return false, nil
			}
		}
		if filter["short"] != false {
			short := Min(width, height)
			if !cmp(short, filter["short"].(int), filter["short_op"].(string)) {
				return false, nil
			}
		}
	}

	return true, nil
}

func main() {
	if len(os.Args) < 3 {
		printUsageAndPanic()
	}

	mode := os.Args[1]
	fmt.Println("mode: ", mode)

	filter = make(map[string]interface{})

	var goal map[string]interface{}
	goal = make(map[string]interface{})

	for _, arg := range os.Args[1 : len(os.Args)-1] {
		not := arg[0] == '!'
		if not {
			arg = arg[1:]
		}

		low := strings.ToLower(arg)
		if low == "valid" || low == "invalid" {
			filter["valid"] = low == "valid"
		} else if low == "rgb" || low == "rgba" || low == "gray" {
			filter["channel"] = low
		} else if low == "png" || low == "jpg" || low == "jpeg" {
			if low == "jpeg" {
				low = "jpg"
			}
			filter["format"] = low
		} else if strings.HasPrefix(low, "filesize") {
			last := arg[len(arg)-1]
			unit := 0
			if last == 'B' {
				unit = 1
			} else if last == 'K' {
				unit = 1 << 3
			} else if last == 'M' {
				unit = 1 << 6
			} else {
				fmt.Println("filesize requires unit (B/K/M)")
				os.Exit(1)
			}
			f, err := strconv.ParseFloat(arg[10:len(arg)-1], 64)
			if err != nil {
				fmt.Println("filesize argument  error", err)
				os.Exit(1)
			}
			filter["filesize"] = int(f) * unit
			filter["filesize_op"] = arg[9]
		} else if strings.HasPrefix(low, "width") ||
			strings.HasPrefix(low, "height") ||
			strings.HasPrefix(low, "long") ||
			strings.HasPrefix(low, "short") {

			r := regexp.MustCompile(`(width|height|long|short)(=|==|\>|\<|\>=|\<=)(\d+)`)
			m := r.FindStringSubmatch(low)
			if len(m) < 4 {
				fmt.Println("dim argument is wrong")
				os.Exit(1)
			}
			attr := m[1]
			dim, err := strconv.ParseInt(m[3], 10, 32)
			if err != nil {
				fmt.Println("dim argument is wrong: cannot parse number")
				os.Exit(1)
			}
			op := m[2]
			if op == "=" {
				goal[attr] = dim
			} else {
				filter[attr] = dim
				filter[attr+"_op"] = op
			}
		}
	}

	fmt.Printf("filter: %+v\n", filter)
	fmt.Printf("goal: %+v\n", goal)

	srcDir := os.Args[len(os.Args)-1]
	src := []string{}
	err := godirwalk.Walk(srcDir, &godirwalk.Options{
		Callback: func(path string, de *godirwalk.Dirent) error {
			src = append(src, path)
			return nil
		},
	})
	if err != nil {
		fmt.Println("error reading directory: ", err)
		os.Exit(1)
	}

	var works errgroup.Group
	defer works.Wait()

	switch os.Args[1] {
	case "any":
	case "list":
		for _, path := range src {
			go func(path string) {
				var lock sync.Mutex
				lock.Lock()
				fmt.Println(path)
				lock.Unlock()
			}(path)
		}
	case "count":
	case "remove":
	case "convert":
	case "move":
	case "copy":
	}
}
