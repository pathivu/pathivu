/*
 * Copyright 2019 Balaji Jinnah and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"misc/api"

	"google.golang.org/grpc"
)

func main() {
	conn, err := grpc.Dial("localhost:6180", grpc.WithInsecure(), grpc.WithBlock())
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	pathivuClient := api.NewPathivuClient(conn)

	data1, _ := json.Marshal(map[string]string{"country": "india"})
	data2, _ := json.Marshal(map[string]string{"country": "pakistan"})
	req := &api.PushRequest{
		Source: "demo",
		Lines: []*api.PushLogLine{
			&api.PushLogLine{
				Ts:         1,
				RawData:    data1,
				Structured: true,
			},
			&api.PushLogLine{
				Ts:         2,
				RawData:    data2,
				Structured: true,
			},
		},
	}

	_, err = pathivuClient.Push(context.Background(), req)
	fmt.Println(err)
	if err != nil {
		log.Fatal(err)
	}
}
