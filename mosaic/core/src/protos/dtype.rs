// This file is generated by rust-protobuf 2.27.1. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `protos/dtype.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_27_1;

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum DataType {
    DT_INVALID = 0,
    DT_F16 = 1,
    DT_F32 = 2,
    DT_F64 = 3,
    DT_INT8 = 4,
    DT_INT16 = 5,
    DT_INT32 = 6,
    DT_INT64 = 7,
    DT_UINT8 = 8,
    DT_UINT16 = 9,
    DT_UINT32 = 10,
    DT_UINT64 = 11,
    DT_STRING = 12,
}

impl ::protobuf::ProtobufEnum for DataType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<DataType> {
        match value {
            0 => ::std::option::Option::Some(DataType::DT_INVALID),
            1 => ::std::option::Option::Some(DataType::DT_F16),
            2 => ::std::option::Option::Some(DataType::DT_F32),
            3 => ::std::option::Option::Some(DataType::DT_F64),
            4 => ::std::option::Option::Some(DataType::DT_INT8),
            5 => ::std::option::Option::Some(DataType::DT_INT16),
            6 => ::std::option::Option::Some(DataType::DT_INT32),
            7 => ::std::option::Option::Some(DataType::DT_INT64),
            8 => ::std::option::Option::Some(DataType::DT_UINT8),
            9 => ::std::option::Option::Some(DataType::DT_UINT16),
            10 => ::std::option::Option::Some(DataType::DT_UINT32),
            11 => ::std::option::Option::Some(DataType::DT_UINT64),
            12 => ::std::option::Option::Some(DataType::DT_STRING),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [DataType] = &[
            DataType::DT_INVALID,
            DataType::DT_F16,
            DataType::DT_F32,
            DataType::DT_F64,
            DataType::DT_INT8,
            DataType::DT_INT16,
            DataType::DT_INT32,
            DataType::DT_INT64,
            DataType::DT_UINT8,
            DataType::DT_UINT16,
            DataType::DT_UINT32,
            DataType::DT_UINT64,
            DataType::DT_STRING,
        ];
        values
    }

    fn enum_descriptor_static() -> &'static ::protobuf::reflect::EnumDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::EnumDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            ::protobuf::reflect::EnumDescriptor::new_pb_name::<DataType>("DataType", file_descriptor_proto())
        })
    }
}

impl ::std::marker::Copy for DataType {
}

impl ::std::default::Default for DataType {
    fn default() -> Self {
        DataType::DT_INVALID
    }
}

impl ::protobuf::reflect::ProtobufValue for DataType {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Enum(::protobuf::ProtobufEnum::descriptor(self))
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x12protos/dtype.proto\x12\rmosaic.protos*\xbf\x01\n\x08DataType\x12\
    \x0e\n\nDT_INVALID\x10\0\x12\n\n\x06DT_F16\x10\x01\x12\n\n\x06DT_F32\x10\
    \x02\x12\n\n\x06DT_F64\x10\x03\x12\x0b\n\x07DT_INT8\x10\x04\x12\x0c\n\
    \x08DT_INT16\x10\x05\x12\x0c\n\x08DT_INT32\x10\x06\x12\x0c\n\x08DT_INT64\
    \x10\x07\x12\x0c\n\x08DT_UINT8\x10\x08\x12\r\n\tDT_UINT16\x10\t\x12\r\n\
    \tDT_UINT32\x10\n\x12\r\n\tDT_UINT64\x10\x0b\x12\r\n\tDT_STRING\x10\x0cb\
    \x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
