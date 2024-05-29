// Copyright 2018 Venil Noronha. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"io"
	"net"
	"os"
	"time"
)

var (
	CRLF              = "\r\n"
	port              int
	deley             int
	level             int
	path              string
	returnValue                     = []byte("-->OK!")
	httpProtcolHeader               = []byte("GET / HTTP/1")
	notfound                        = []byte{}
	sleepDuration     time.Duration = 0
)

func status_fn(code int32, text string) string {
	return fmt.Sprintf("HTTP/1.1 %v %v%v", code, text, CRLF)
}

func init() {
	flag.StringVar(&path, "path", "", "tcp返回值路径，可以手动设置为 http协议，默认返回'-->OK!'")
	flag.IntVar(&port, "port", 8080, "绑定端口, 默认8080")
	flag.IntVar(&deley, "deley", 0, "每个请求延时，默认0")
	flag.IntVar(&level, "level", 4, "网络协议层 4 or 7， 默认4，请求完不会主动断开，需要client断开")
}

func IsFile(f string) bool {
	fi, e := os.Stat(f)
	if e != nil {
		return false
	}
	return !fi.IsDir()
}

// main serves as the program entry point
func main() {

	flag.Parse()

	if path != "" {
		content, err := os.ReadFile(path)
		if err != nil {
			fmt.Printf("[Error:] %v\n", err)
			os.Exit(1)
		}

		returnValue = []byte(content)

		// if IsFile(path) {

		// } else {
		// 	fmt.Println("The filepath isn't a file or Not Exist.")
		// 	os.Exit(1)
		// }
	}
	fmt.Printf("返回输入内容为%s\n", returnValue)
	fmt.Printf("最后一个字符十进制为%v，如果使用jmeter tcp压测，请设置为结束位\n", returnValue[len(returnValue)-1])
	if deley != 0 {
		sleepDuration = time.Millisecond * time.Duration(deley)
	}
	if level == 7 {

		var content_type = fmt.Sprintf("Content-Type: text/html;charset=utf-8%v", CRLF)
		var server = fmt.Sprintf("Server: Golang%v", CRLF)
		var content_length = fmt.Sprintf("Content-Length: %v%v", len(returnValue), CRLF)
		// var mut response: &str = "";
		var status200 = status_fn(200, "OK")
		var status404 = status_fn(404, "NOT FOUND")
		returnValue = []byte(fmt.Sprintf("%v%v%v%v%v%s", status200, server, content_type, content_length, CRLF, returnValue))
		notfound = []byte(fmt.Sprintf("%v%v%v%v%v", status404, server, content_type, fmt.Sprintf("Content-Length: %v%v", 0, CRLF), CRLF))
		// fmt.Printf("%s\n%s", returnValue, notfound)
	} else if level == 4 {

	} else {
		fmt.Printf("四层or七层？: %d\n", level)
		os.Exit(1)
	}

	fmt.Printf("每个请求延时时间:%v\n", sleepDuration)
	fmt.Printf("四层or七层？: %d\n", level)
	fmt.Printf("请求响应内容路径:%s\n", path)
	fmt.Printf("监听端口:%d\n", port)

	// obtain the port and prefix via program arguments
	// port := fmt.Sprintf(":%s", os.Args[1])
	// prefix := os.Args[2]

	// create a tcp listener on the given port
	listener, err := net.Listen("tcp4", fmt.Sprintf("0.0.0.0:%d", port))
	if err != nil {
		fmt.Println("failed to create listener, err:", err)
		os.Exit(1)
	}
	fmt.Printf("listening on %s, prefix: %s\n", listener.Addr(), "prefix")

	// listen for new connections
	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("failed to accept connection, err:", err)
			continue
		}
		// pass an accepted connection to a handler goroutine
		go handleConnection(conn)
	}
}

// handleConnection handles the lifetime of a connection
func handleConnection(conn net.Conn) {
	defer conn.Close()
	// conn.SetReadDeadline(time.Now().Add(time.Second))
	reader := bufio.NewReader(conn)
	// var p = [1500]byte{}
	var p [1400]byte // 这里如果写，var p []byte，读的时候就不会阻塞！！！！！！！！！！！！！！！！
	for {
		// read client request data
		// conn.Read()
		// conn.Read
		n, err := reader.Read(p[:])
		// bytes, err := reader.ReadBytes(byte('\n'))
		if err != nil || n == 0 {
			if err != io.EOF {
				// fmt.Println("failed to read data, err:", err)
			}
			// fmt.Printf("[%v]\n", err)
			return
		}
		if deley != 0 {
			time.Sleep(sleepDuration)
		}
		if level == 4 {
			conn.Write(returnValue)
		}
		if level == 7 {
			if bytes.Equal(p[:12], httpProtcolHeader) {
				conn.Write(returnValue)
			} else {
				conn.Write(notfound)
			}
		}

		// if len(p[:bytes]) == 15 {
		// p[1000] = byte(bytes)
		// }
		// fmt.Printf("request: %d\n", bytes)

		// prepend prefix and send as response
		// line := fmt.Sprintf("%s %s", prefix, bytes)
		// fmt.Printf("response: %s", line)

	}
}
