#!/bin/bash

# /*
#  * Copyright 2019 Balaji Jinnah and Contributors
#  *
#  * Licensed under the Apache License, Version 2.0 (the "License");
#  * you may not use this file except in compliance with the License.
#  * You may obtain a copy of the License at
#  *
#  *     http://www.apache.org/licenses/LICENSE-2.0
#  *
#  * Unless required by applicable law or agreed to in writing, software
#  * distributed under the License is distributed on an "AS IS" BASIS,
#  * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  * See the License for the specific language governing permissions and
#  * limitations under the License.
#  */

# You might need to go get -v github.com/gogo/protobuf/...

protoc --go_out=plugins=grpc:. api.proto

cp api.pb.go ../katchi/api
cp api.pb.go ../misc/api
rm api.pb.go

# generate for ruby
grpc_tools_ruby_protoc -I . --ruby_out=. --grpc_out=. api.proto 
mv api_pb.rb ../fluentd-plugin/debian-chola-plugins
mv api_service_pb.rb ../fluentd-plugin/debian-chola-plugins