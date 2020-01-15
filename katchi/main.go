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
	"fmt"
	"katchi/api"
	"log"
	"os"
	"time"

	"github.com/spf13/cobra"
	"github.com/tidwall/pretty"
	"google.golang.org/grpc"
)

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

func printLogs(lines []*api.LogLine) {
	for _, line := range lines {
		fmt.Printf("APP: %s, ts: %s, line: %s \n", line.App, time.Unix(int64(line.Ts), 0).String(),
			line.Inner)
	}
}

type client struct {
	conn *grpc.ClientConn
}

func newClient(host string) *client {
	conn, err := grpc.Dial(host, grpc.WithInsecure(), grpc.WithBlock())
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	return &client{
		conn: conn,
	}
}

func (c *client) CloseConn() error {
	return c.conn.Close()
}

func (c *client) partitions() []string {
	pathivuClient := api.NewPathivuClient(c.conn)
	res, err := pathivuClient.Partitions(context.Background(), &api.Empty{})
	if err != nil {
		log.Fatal(err)
	}
	return res.Partitions
}

func (c *client) query(req *api.QueryRequest) *api.QueryResponse {
	pathivuClient := api.NewPathivuClient(c.conn)
	res, err := pathivuClient.Query(context.Background(), req)
	if err != nil {
		log.Fatal(err)
	}
	return res
}

func (c *client) tail(partitions []string) {
	req := &api.QueryRequest{
		Partitions: partitions,
	}
	pathivuClient := api.NewPathivuClient(c.conn)
	stream, err := pathivuClient.Tail(context.Background(), req)
	if err != nil {
		log.Fatal(err)
	}
	for {
		res, err := stream.Recv()
		if err != nil {
			log.Fatal(err)
		}
		printLogs(res.Lines)
	}
}

var host string

var req *api.QueryRequest

var since time.Duration

var partitions []string

var startTs int64

var endTs int64

func main() {
	req = new(api.QueryRequest)
	since = (time.Second * 0)
	rootCmd := &cobra.Command{
		Use:   "katchi",
		Short: "Katchi is log visualization tool for pathivu",
		Long:  `katch is a cli tool for pathivu, which let's you folks to view all the logs in pathivu`,
	}
	var queryCmd = &cobra.Command{
		Use:   "logs --apps=kube-server --partitions=api-server --since=3h --query=info",
		Short: "query logs ",
		Long:  `query logs in the pathivu server`,
		Run: func(cmd *cobra.Command, args []string) {
			if host == "" {
				log.Fatalf("host name required. Set pathivu_HOST env or pass host flag")
			}
			c := newClient(host)
			defer c.CloseConn()
			req.Partitions = partitions
			if since.Seconds() != 0 {
				req.StartTs = uint64(time.Now().Add(-since).Unix())
				req.EndTs = uint64(time.Now().Unix())
			}
			req.Forward = false
			res := c.query(req)
			fmt.Println(string(pretty.Pretty([]byte(res.Json))))
		},
	}
	queryCmd.Flags().DurationVar(&since, "since", time.Second*0, "since=1h")
	queryCmd.Flags().StringArrayVar(&partitions, "apps", []string{}, "apps=kubeserver")

	var tailCmd = &cobra.Command{
		Use:   "tail --apps=kube-server  --apps=api-server",
		Short: "tail all the apps",
		Long:  "tail logs of the specified apps",
		Run: func(cmd *cobra.Command, args []string) {
			if host == "" {
				log.Fatalf("host name required. Set pathivu_HOST env or pass host flag")
			}
			c := newClient(host)
			defer c.CloseConn()
			c.tail(partitions)
		},
	}
	tailCmd.Flags().StringArrayVar(&partitions, "apps", []string{}, "apps=kubeserver")

	partitionCmd := &cobra.Command{
		Use:   "apps",
		Short: "apps gives all the app name of logs that has been ingested in the pathivu",
		Long:  `apps gives all the app name of logs that has been ingested in the pathivu`,
		Run: func(cmd *cobra.Command, args []string) {
			if host == "" {
				log.Fatalf("host name required. Set pathivu_HOST env or pass host flag")
			}
			c := newClient(host)
			defer c.CloseConn()
			partitons := c.partitions()
			fmt.Println("App Names")
			for _, app := range partitons {
				fmt.Println(app)
			}
		},
	}
	rootCmd.PersistentFlags().StringVar(&host, "host", os.Getenv("pathivu_HOST"), "pathivu host address")
	rootCmd.AddCommand(queryCmd)
	rootCmd.AddCommand(partitionCmd)
	rootCmd.AddCommand(tailCmd)
	rootCmd.Execute()
}
