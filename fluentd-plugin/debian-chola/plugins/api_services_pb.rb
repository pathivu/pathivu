# Generated by the protocol buffer compiler.  DO NOT EDIT!
# Source: api.proto for package 'api'
# Original file comments:
#
# Copyright 2019 Balaji Jinnah and Contributors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

require 'grpc'
require_relative 'api_pb'

module Api
  module Pathivu
    # Interface exported by the server.
    class Service

      include GRPC::GenericService

      self.marshal_class_method = :encode
      self.unmarshal_class_method = :decode
      self.service_name = 'api.Pathivu'

      # Tail 
      rpc :Tail, QueryRequest, stream(QueryResponse)
      # Query
      rpc :Query, QueryRequest, QueryResponse
      # Partitions
      rpc :Partitions, Empty, PartitionResponse
      # Push
      rpc :Push, PushRequest, Empty
    end

    Stub = Service.rpc_stub_class
  end
end
