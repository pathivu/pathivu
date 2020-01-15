// Code generated by protoc-gen-go. DO NOT EDIT.
// source: api.proto

package api

import (
	context "context"
	fmt "fmt"
	proto "github.com/golang/protobuf/proto"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
	math "math"
)

// Reference imports to suppress errors if they are not otherwise used.
var _ = proto.Marshal
var _ = fmt.Errorf
var _ = math.Inf

// This is a compile-time assertion to ensure that this generated file
// is compatible with the proto package it is being compiled against.
// A compilation error at this line likely means your copy of the
// proto package needs to be updated.
const _ = proto.ProtoPackageIsVersion3 // please upgrade the proto package

type QueryRequest struct {
	Partitions           []string `protobuf:"bytes,1,rep,name=partitions,proto3" json:"partitions,omitempty"`
	StartTs              uint64   `protobuf:"varint,2,opt,name=start_ts,json=startTs,proto3" json:"start_ts,omitempty"`
	EndTs                uint64   `protobuf:"varint,3,opt,name=end_ts,json=endTs,proto3" json:"end_ts,omitempty"`
	Count                uint64   `protobuf:"varint,4,opt,name=count,proto3" json:"count,omitempty"`
	Offset               uint64   `protobuf:"varint,5,opt,name=offset,proto3" json:"offset,omitempty"`
	Forward              bool     `protobuf:"varint,6,opt,name=forward,proto3" json:"forward,omitempty"`
	Query                string   `protobuf:"bytes,7,opt,name=query,proto3" json:"query,omitempty"`
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *QueryRequest) Reset()         { *m = QueryRequest{} }
func (m *QueryRequest) String() string { return proto.CompactTextString(m) }
func (*QueryRequest) ProtoMessage()    {}
func (*QueryRequest) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{0}
}

func (m *QueryRequest) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_QueryRequest.Unmarshal(m, b)
}
func (m *QueryRequest) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_QueryRequest.Marshal(b, m, deterministic)
}
func (m *QueryRequest) XXX_Merge(src proto.Message) {
	xxx_messageInfo_QueryRequest.Merge(m, src)
}
func (m *QueryRequest) XXX_Size() int {
	return xxx_messageInfo_QueryRequest.Size(m)
}
func (m *QueryRequest) XXX_DiscardUnknown() {
	xxx_messageInfo_QueryRequest.DiscardUnknown(m)
}

var xxx_messageInfo_QueryRequest proto.InternalMessageInfo

func (m *QueryRequest) GetPartitions() []string {
	if m != nil {
		return m.Partitions
	}
	return nil
}

func (m *QueryRequest) GetStartTs() uint64 {
	if m != nil {
		return m.StartTs
	}
	return 0
}

func (m *QueryRequest) GetEndTs() uint64 {
	if m != nil {
		return m.EndTs
	}
	return 0
}

func (m *QueryRequest) GetCount() uint64 {
	if m != nil {
		return m.Count
	}
	return 0
}

func (m *QueryRequest) GetOffset() uint64 {
	if m != nil {
		return m.Offset
	}
	return 0
}

func (m *QueryRequest) GetForward() bool {
	if m != nil {
		return m.Forward
	}
	return false
}

func (m *QueryRequest) GetQuery() string {
	if m != nil {
		return m.Query
	}
	return ""
}

type QueryResponse struct {
	Lines                []*LogLine `protobuf:"bytes,1,rep,name=lines,proto3" json:"lines,omitempty"`
	Json                 string     `protobuf:"bytes,2,opt,name=json,proto3" json:"json,omitempty"`
	XXX_NoUnkeyedLiteral struct{}   `json:"-"`
	XXX_unrecognized     []byte     `json:"-"`
	XXX_sizecache        int32      `json:"-"`
}

func (m *QueryResponse) Reset()         { *m = QueryResponse{} }
func (m *QueryResponse) String() string { return proto.CompactTextString(m) }
func (*QueryResponse) ProtoMessage()    {}
func (*QueryResponse) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{1}
}

func (m *QueryResponse) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_QueryResponse.Unmarshal(m, b)
}
func (m *QueryResponse) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_QueryResponse.Marshal(b, m, deterministic)
}
func (m *QueryResponse) XXX_Merge(src proto.Message) {
	xxx_messageInfo_QueryResponse.Merge(m, src)
}
func (m *QueryResponse) XXX_Size() int {
	return xxx_messageInfo_QueryResponse.Size(m)
}
func (m *QueryResponse) XXX_DiscardUnknown() {
	xxx_messageInfo_QueryResponse.DiscardUnknown(m)
}

var xxx_messageInfo_QueryResponse proto.InternalMessageInfo

func (m *QueryResponse) GetLines() []*LogLine {
	if m != nil {
		return m.Lines
	}
	return nil
}

func (m *QueryResponse) GetJson() string {
	if m != nil {
		return m.Json
	}
	return ""
}

type LogLine struct {
	Inner                string   `protobuf:"bytes,1,opt,name=inner,proto3" json:"inner,omitempty"`
	Ts                   uint64   `protobuf:"varint,2,opt,name=ts,proto3" json:"ts,omitempty"`
	App                  string   `protobuf:"bytes,3,opt,name=app,proto3" json:"app,omitempty"`
	Structured           bool     `protobuf:"varint,4,opt,name=structured,proto3" json:"structured,omitempty"`
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *LogLine) Reset()         { *m = LogLine{} }
func (m *LogLine) String() string { return proto.CompactTextString(m) }
func (*LogLine) ProtoMessage()    {}
func (*LogLine) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{2}
}

func (m *LogLine) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_LogLine.Unmarshal(m, b)
}
func (m *LogLine) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_LogLine.Marshal(b, m, deterministic)
}
func (m *LogLine) XXX_Merge(src proto.Message) {
	xxx_messageInfo_LogLine.Merge(m, src)
}
func (m *LogLine) XXX_Size() int {
	return xxx_messageInfo_LogLine.Size(m)
}
func (m *LogLine) XXX_DiscardUnknown() {
	xxx_messageInfo_LogLine.DiscardUnknown(m)
}

var xxx_messageInfo_LogLine proto.InternalMessageInfo

func (m *LogLine) GetInner() string {
	if m != nil {
		return m.Inner
	}
	return ""
}

func (m *LogLine) GetTs() uint64 {
	if m != nil {
		return m.Ts
	}
	return 0
}

func (m *LogLine) GetApp() string {
	if m != nil {
		return m.App
	}
	return ""
}

func (m *LogLine) GetStructured() bool {
	if m != nil {
		return m.Structured
	}
	return false
}

type PartitionResponse struct {
	Partitions           []string `protobuf:"bytes,1,rep,name=partitions,proto3" json:"partitions,omitempty"`
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *PartitionResponse) Reset()         { *m = PartitionResponse{} }
func (m *PartitionResponse) String() string { return proto.CompactTextString(m) }
func (*PartitionResponse) ProtoMessage()    {}
func (*PartitionResponse) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{3}
}

func (m *PartitionResponse) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_PartitionResponse.Unmarshal(m, b)
}
func (m *PartitionResponse) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_PartitionResponse.Marshal(b, m, deterministic)
}
func (m *PartitionResponse) XXX_Merge(src proto.Message) {
	xxx_messageInfo_PartitionResponse.Merge(m, src)
}
func (m *PartitionResponse) XXX_Size() int {
	return xxx_messageInfo_PartitionResponse.Size(m)
}
func (m *PartitionResponse) XXX_DiscardUnknown() {
	xxx_messageInfo_PartitionResponse.DiscardUnknown(m)
}

var xxx_messageInfo_PartitionResponse proto.InternalMessageInfo

func (m *PartitionResponse) GetPartitions() []string {
	if m != nil {
		return m.Partitions
	}
	return nil
}

type Empty struct {
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *Empty) Reset()         { *m = Empty{} }
func (m *Empty) String() string { return proto.CompactTextString(m) }
func (*Empty) ProtoMessage()    {}
func (*Empty) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{4}
}

func (m *Empty) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_Empty.Unmarshal(m, b)
}
func (m *Empty) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_Empty.Marshal(b, m, deterministic)
}
func (m *Empty) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Empty.Merge(m, src)
}
func (m *Empty) XXX_Size() int {
	return xxx_messageInfo_Empty.Size(m)
}
func (m *Empty) XXX_DiscardUnknown() {
	xxx_messageInfo_Empty.DiscardUnknown(m)
}

var xxx_messageInfo_Empty proto.InternalMessageInfo

type PushLogLine struct {
	Ts                   uint64   `protobuf:"varint,1,opt,name=ts,proto3" json:"ts,omitempty"`
	Indexes              []string `protobuf:"bytes,2,rep,name=indexes,proto3" json:"indexes,omitempty"`
	Structured           bool     `protobuf:"varint,3,opt,name=structured,proto3" json:"structured,omitempty"`
	RawData              []byte   `protobuf:"bytes,4,opt,name=raw_data,json=rawData,proto3" json:"raw_data,omitempty"`
	JsonKeys             []string `protobuf:"bytes,5,rep,name=json_keys,json=jsonKeys,proto3" json:"json_keys,omitempty"`
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *PushLogLine) Reset()         { *m = PushLogLine{} }
func (m *PushLogLine) String() string { return proto.CompactTextString(m) }
func (*PushLogLine) ProtoMessage()    {}
func (*PushLogLine) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{5}
}

func (m *PushLogLine) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_PushLogLine.Unmarshal(m, b)
}
func (m *PushLogLine) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_PushLogLine.Marshal(b, m, deterministic)
}
func (m *PushLogLine) XXX_Merge(src proto.Message) {
	xxx_messageInfo_PushLogLine.Merge(m, src)
}
func (m *PushLogLine) XXX_Size() int {
	return xxx_messageInfo_PushLogLine.Size(m)
}
func (m *PushLogLine) XXX_DiscardUnknown() {
	xxx_messageInfo_PushLogLine.DiscardUnknown(m)
}

var xxx_messageInfo_PushLogLine proto.InternalMessageInfo

func (m *PushLogLine) GetTs() uint64 {
	if m != nil {
		return m.Ts
	}
	return 0
}

func (m *PushLogLine) GetIndexes() []string {
	if m != nil {
		return m.Indexes
	}
	return nil
}

func (m *PushLogLine) GetStructured() bool {
	if m != nil {
		return m.Structured
	}
	return false
}

func (m *PushLogLine) GetRawData() []byte {
	if m != nil {
		return m.RawData
	}
	return nil
}

func (m *PushLogLine) GetJsonKeys() []string {
	if m != nil {
		return m.JsonKeys
	}
	return nil
}

type PushRequest struct {
	Source               string         `protobuf:"bytes,1,opt,name=source,proto3" json:"source,omitempty"`
	Lines                []*PushLogLine `protobuf:"bytes,2,rep,name=lines,proto3" json:"lines,omitempty"`
	XXX_NoUnkeyedLiteral struct{}       `json:"-"`
	XXX_unrecognized     []byte         `json:"-"`
	XXX_sizecache        int32          `json:"-"`
}

func (m *PushRequest) Reset()         { *m = PushRequest{} }
func (m *PushRequest) String() string { return proto.CompactTextString(m) }
func (*PushRequest) ProtoMessage()    {}
func (*PushRequest) Descriptor() ([]byte, []int) {
	return fileDescriptor_00212fb1f9d3bf1c, []int{6}
}

func (m *PushRequest) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_PushRequest.Unmarshal(m, b)
}
func (m *PushRequest) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_PushRequest.Marshal(b, m, deterministic)
}
func (m *PushRequest) XXX_Merge(src proto.Message) {
	xxx_messageInfo_PushRequest.Merge(m, src)
}
func (m *PushRequest) XXX_Size() int {
	return xxx_messageInfo_PushRequest.Size(m)
}
func (m *PushRequest) XXX_DiscardUnknown() {
	xxx_messageInfo_PushRequest.DiscardUnknown(m)
}

var xxx_messageInfo_PushRequest proto.InternalMessageInfo

func (m *PushRequest) GetSource() string {
	if m != nil {
		return m.Source
	}
	return ""
}

func (m *PushRequest) GetLines() []*PushLogLine {
	if m != nil {
		return m.Lines
	}
	return nil
}

func init() {
	proto.RegisterType((*QueryRequest)(nil), "api.QueryRequest")
	proto.RegisterType((*QueryResponse)(nil), "api.QueryResponse")
	proto.RegisterType((*LogLine)(nil), "api.LogLine")
	proto.RegisterType((*PartitionResponse)(nil), "api.PartitionResponse")
	proto.RegisterType((*Empty)(nil), "api.Empty")
	proto.RegisterType((*PushLogLine)(nil), "api.PushLogLine")
	proto.RegisterType((*PushRequest)(nil), "api.PushRequest")
}

func init() { proto.RegisterFile("api.proto", fileDescriptor_00212fb1f9d3bf1c) }

var fileDescriptor_00212fb1f9d3bf1c = []byte{
	// 471 bytes of a gzipped FileDescriptorProto
	0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0x8c, 0x93, 0xcd, 0x6e, 0x13, 0x3f,
	0x14, 0xc5, 0xe3, 0xcc, 0x57, 0xe6, 0x36, 0xff, 0xbf, 0xda, 0x2b, 0x88, 0x4c, 0x91, 0x50, 0xe4,
	0x45, 0x95, 0x55, 0x55, 0xd2, 0x57, 0x00, 0xb1, 0xa0, 0x48, 0xc1, 0xca, 0x3e, 0x32, 0x19, 0x87,
	0x1a, 0x8a, 0x3d, 0xb5, 0x3d, 0x84, 0x3c, 0x05, 0x6f, 0xc4, 0x03, 0xf0, 0x54, 0xc8, 0x9e, 0x99,
	0x74, 0x94, 0x0d, 0xec, 0xe6, 0x5c, 0x7f, 0xfd, 0xce, 0xb9, 0x77, 0xa0, 0x14, 0xb5, 0xba, 0xae,
	0xad, 0xf1, 0x06, 0x13, 0x51, 0x2b, 0xf6, 0x8b, 0xc0, 0xf4, 0x63, 0x23, 0xed, 0x81, 0xcb, 0xc7,
	0x46, 0x3a, 0x8f, 0xaf, 0x00, 0x6a, 0x61, 0xbd, 0xf2, 0xca, 0x68, 0x47, 0xc9, 0x3c, 0x59, 0x94,
	0x7c, 0x50, 0xc1, 0x17, 0x30, 0x71, 0x5e, 0x58, 0xbf, 0xf1, 0x8e, 0x8e, 0xe7, 0x64, 0x91, 0xf2,
	0x22, 0xea, 0xb5, 0xc3, 0xe7, 0x90, 0x4b, 0x5d, 0x85, 0x85, 0x24, 0x2e, 0x64, 0x52, 0x57, 0x6b,
	0x87, 0xcf, 0x20, 0xdb, 0x9a, 0x46, 0x7b, 0x9a, 0xb6, 0xd5, 0x28, 0x70, 0x06, 0xb9, 0xd9, 0xed,
	0x9c, 0xf4, 0x34, 0x8b, 0xe5, 0x4e, 0x21, 0x85, 0x62, 0x67, 0xec, 0x5e, 0xd8, 0x8a, 0xe6, 0x73,
	0xb2, 0x98, 0xf0, 0x5e, 0x86, 0x7b, 0x1e, 0x03, 0x29, 0x2d, 0xe6, 0x64, 0x51, 0xf2, 0x56, 0xb0,
	0x77, 0xf0, 0x5f, 0xc7, 0xef, 0x6a, 0xa3, 0x9d, 0x44, 0x06, 0xd9, 0x83, 0xd2, 0xb2, 0x65, 0x3f,
	0x5b, 0x4e, 0xaf, 0x83, 0xe3, 0x3b, 0xf3, 0xf9, 0x4e, 0x69, 0xc9, 0xdb, 0x25, 0x44, 0x48, 0xbf,
	0x38, 0xa3, 0xa3, 0x81, 0x92, 0xc7, 0x6f, 0x26, 0xa0, 0xe8, 0x76, 0x85, 0x97, 0x94, 0xd6, 0xd2,
	0x52, 0xd2, 0xbe, 0x14, 0x05, 0xfe, 0x0f, 0xe3, 0xa3, 0xe7, 0xb1, 0x77, 0x78, 0x0e, 0x89, 0xa8,
	0xeb, 0xe8, 0xb5, 0xe4, 0xe1, 0x33, 0x64, 0xe7, 0xbc, 0x6d, 0xb6, 0xbe, 0xb1, 0xb2, 0x8a, 0x76,
	0x27, 0x7c, 0x50, 0x61, 0xb7, 0x70, 0xb1, 0xea, 0x93, 0x3c, 0xf2, 0xfe, 0x25, 0x70, 0x56, 0x40,
	0xf6, 0xf6, 0x5b, 0xed, 0x0f, 0xec, 0x27, 0x81, 0xb3, 0x55, 0xe3, 0xee, 0x7b, 0xca, 0x96, 0x87,
	0x1c, 0x79, 0x28, 0x14, 0x4a, 0x57, 0xf2, 0x87, 0x0c, 0x90, 0xe1, 0x96, 0x5e, 0x9e, 0x70, 0x25,
	0xa7, 0x5c, 0xa1, 0xa7, 0x56, 0xec, 0x37, 0x95, 0xf0, 0x22, 0x52, 0x4f, 0x79, 0x61, 0xc5, 0xfe,
	0x8d, 0xf0, 0x02, 0x5f, 0x42, 0x19, 0xd2, 0xd9, 0x7c, 0x95, 0x07, 0x47, 0xb3, 0x78, 0xed, 0x24,
	0x14, 0xde, 0xcb, 0x83, 0x63, 0x1f, 0x5a, 0xa0, 0x7e, 0x74, 0x66, 0x90, 0x3b, 0xd3, 0xd8, 0xad,
	0xec, 0x72, 0xeb, 0x14, 0x5e, 0xf5, 0x1d, 0x19, 0xc7, 0x8e, 0x9c, 0xc7, 0x8e, 0x0c, 0x9c, 0x74,
	0x5d, 0x59, 0xfe, 0x26, 0x50, 0xac, 0x84, 0xbf, 0x57, 0xdf, 0x1b, 0x7c, 0x0d, 0xe9, 0x5a, 0xa8,
	0x07, 0xbc, 0x88, 0x9b, 0x87, 0x13, 0x7a, 0x89, 0xc3, 0x52, 0x1b, 0x22, 0x1b, 0xdd, 0x10, 0xbc,
	0x81, 0x2c, 0x16, 0xff, 0xf9, 0x0c, 0x2e, 0x01, 0x56, 0x4f, 0x93, 0x0d, 0x71, 0x4f, 0xcc, 0xfa,
	0x72, 0xd6, 0x32, 0x9e, 0x36, 0x8b, 0x8d, 0xf0, 0x0a, 0xd2, 0x80, 0x8e, 0x4f, 0x2e, 0xfa, 0x37,
	0x06, 0xe7, 0xd9, 0xe8, 0x53, 0x1e, 0x7f, 0xb2, 0xdb, 0x3f, 0x01, 0x00, 0x00, 0xff, 0xff, 0x6c,
	0xc8, 0xff, 0x5b, 0x71, 0x03, 0x00, 0x00,
}

// Reference imports to suppress errors if they are not otherwise used.
var _ context.Context
var _ grpc.ClientConn

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
const _ = grpc.SupportPackageIsVersion4

// PathivuClient is the client API for Pathivu service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://godoc.org/google.golang.org/grpc#ClientConn.NewStream.
type PathivuClient interface {
	// Tail
	Tail(ctx context.Context, in *QueryRequest, opts ...grpc.CallOption) (Pathivu_TailClient, error)
	// Query
	Query(ctx context.Context, in *QueryRequest, opts ...grpc.CallOption) (*QueryResponse, error)
	// Partitions
	Partitions(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*PartitionResponse, error)
	// Push
	Push(ctx context.Context, in *PushRequest, opts ...grpc.CallOption) (*Empty, error)
}

type pathivuClient struct {
	cc *grpc.ClientConn
}

func NewPathivuClient(cc *grpc.ClientConn) PathivuClient {
	return &pathivuClient{cc}
}

func (c *pathivuClient) Tail(ctx context.Context, in *QueryRequest, opts ...grpc.CallOption) (Pathivu_TailClient, error) {
	stream, err := c.cc.NewStream(ctx, &_Pathivu_serviceDesc.Streams[0], "/api.Pathivu/Tail", opts...)
	if err != nil {
		return nil, err
	}
	x := &pathivuTailClient{stream}
	if err := x.ClientStream.SendMsg(in); err != nil {
		return nil, err
	}
	if err := x.ClientStream.CloseSend(); err != nil {
		return nil, err
	}
	return x, nil
}

type Pathivu_TailClient interface {
	Recv() (*QueryResponse, error)
	grpc.ClientStream
}

type pathivuTailClient struct {
	grpc.ClientStream
}

func (x *pathivuTailClient) Recv() (*QueryResponse, error) {
	m := new(QueryResponse)
	if err := x.ClientStream.RecvMsg(m); err != nil {
		return nil, err
	}
	return m, nil
}

func (c *pathivuClient) Query(ctx context.Context, in *QueryRequest, opts ...grpc.CallOption) (*QueryResponse, error) {
	out := new(QueryResponse)
	err := c.cc.Invoke(ctx, "/api.Pathivu/Query", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *pathivuClient) Partitions(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*PartitionResponse, error) {
	out := new(PartitionResponse)
	err := c.cc.Invoke(ctx, "/api.Pathivu/Partitions", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *pathivuClient) Push(ctx context.Context, in *PushRequest, opts ...grpc.CallOption) (*Empty, error) {
	out := new(Empty)
	err := c.cc.Invoke(ctx, "/api.Pathivu/Push", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// PathivuServer is the server API for Pathivu service.
type PathivuServer interface {
	// Tail
	Tail(*QueryRequest, Pathivu_TailServer) error
	// Query
	Query(context.Context, *QueryRequest) (*QueryResponse, error)
	// Partitions
	Partitions(context.Context, *Empty) (*PartitionResponse, error)
	// Push
	Push(context.Context, *PushRequest) (*Empty, error)
}

// UnimplementedPathivuServer can be embedded to have forward compatible implementations.
type UnimplementedPathivuServer struct {
}

func (*UnimplementedPathivuServer) Tail(req *QueryRequest, srv Pathivu_TailServer) error {
	return status.Errorf(codes.Unimplemented, "method Tail not implemented")
}
func (*UnimplementedPathivuServer) Query(ctx context.Context, req *QueryRequest) (*QueryResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Query not implemented")
}
func (*UnimplementedPathivuServer) Partitions(ctx context.Context, req *Empty) (*PartitionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Partitions not implemented")
}
func (*UnimplementedPathivuServer) Push(ctx context.Context, req *PushRequest) (*Empty, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Push not implemented")
}

func RegisterPathivuServer(s *grpc.Server, srv PathivuServer) {
	s.RegisterService(&_Pathivu_serviceDesc, srv)
}

func _Pathivu_Tail_Handler(srv interface{}, stream grpc.ServerStream) error {
	m := new(QueryRequest)
	if err := stream.RecvMsg(m); err != nil {
		return err
	}
	return srv.(PathivuServer).Tail(m, &pathivuTailServer{stream})
}

type Pathivu_TailServer interface {
	Send(*QueryResponse) error
	grpc.ServerStream
}

type pathivuTailServer struct {
	grpc.ServerStream
}

func (x *pathivuTailServer) Send(m *QueryResponse) error {
	return x.ServerStream.SendMsg(m)
}

func _Pathivu_Query_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(QueryRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(PathivuServer).Query(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/api.Pathivu/Query",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(PathivuServer).Query(ctx, req.(*QueryRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Pathivu_Partitions_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(Empty)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(PathivuServer).Partitions(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/api.Pathivu/Partitions",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(PathivuServer).Partitions(ctx, req.(*Empty))
	}
	return interceptor(ctx, in, info, handler)
}

func _Pathivu_Push_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(PushRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(PathivuServer).Push(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/api.Pathivu/Push",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(PathivuServer).Push(ctx, req.(*PushRequest))
	}
	return interceptor(ctx, in, info, handler)
}

var _Pathivu_serviceDesc = grpc.ServiceDesc{
	ServiceName: "api.Pathivu",
	HandlerType: (*PathivuServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Query",
			Handler:    _Pathivu_Query_Handler,
		},
		{
			MethodName: "Partitions",
			Handler:    _Pathivu_Partitions_Handler,
		},
		{
			MethodName: "Push",
			Handler:    _Pathivu_Push_Handler,
		},
	},
	Streams: []grpc.StreamDesc{
		{
			StreamName:    "Tail",
			Handler:       _Pathivu_Tail_Handler,
			ServerStreams: true,
		},
	},
	Metadata: "api.proto",
}
