// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.49.0.
// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, prefer_single_quotes, prefer_const_constructors, use_super_parameters, always_use_package_imports, annotate_overrides, invalid_use_of_protected_member, constant_identifier_names

import 'dart:convert';
import 'dart:async';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:uuid/uuid.dart';

import 'package:meta/meta.dart';
import 'package:meta/meta.dart';
import 'dart:ffi' as ffi;

late final ApiClass1Platform inner_platform;

abstract class ApiClass1 {
  /// Documentation on a simple adder function.
  Future<int> simpleAdder1({required int a, required int b, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSimpleAdder1ConstMeta;
}

class ApiClass1Impl implements ApiClass1 {
  final ApiClass1Platform _platform;
  factory ApiClass1Impl(ExternalLibrary dylib) => ApiClass1Impl.raw(ApiClass1Platform(dylib));

  /// Only valid on web/WASM platforms.
  factory ApiClass1Impl.wasm(FutureOr<WasmModule> module) => ApiClass1Impl(module as ExternalLibrary);
  ApiClass1Impl.raw(this._platform) {
    inner_platform = _platform;
  }
  Future<int> simpleAdder1({required int a, required int b, dynamic hint}) {
    var arg0 = api2wire_i32(a);
    var arg1 = api2wire_i32(b);
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_simple_adder_1(port_, arg0, arg1),
      parseSuccessData: _wire2api_i32,
      constMeta: kSimpleAdder1ConstMeta,
      argValues: [a, b],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kSimpleAdder1ConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "simple_adder_1",
        argNames: ["a", "b"],
      );

// Section: wire2api

  int _wire2api_i32(dynamic raw) {
    return raw as int;
  }
}

// Section: api2wire

@protected
int api2wire_i32(int raw) {
  return raw;
}
// Section: finalyzer

class ApiClass1Platform extends FlutterRustBridgeBase<ApiClass1Wire> {
  ApiClass1Platform(ffi.DynamicLibrary dylib) : super(ApiClass1Wire(dylib));
// Section: api2wire

// Section: finalyzer

// Section: api_fill_to_wire

}

// ignore_for_file: camel_case_types, non_constant_identifier_names, avoid_positional_boolean_parameters, annotate_overrides, constant_identifier_names

// AUTO GENERATED FILE, DO NOT EDIT.
//
// Generated by `package:ffigen`.

/// generated by flutter_rust_bridge
class ApiClass1Wire implements FlutterRustBridgeWireBase {
  /// Holds the symbol lookup function.
  final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName) _lookup;

  /// The symbols are looked up in [dynamicLibrary].
  ApiClass1Wire(ffi.DynamicLibrary dynamicLibrary) : _lookup = dynamicLibrary.lookup;

  /// The symbols are looked up with [lookup].
  ApiClass1Wire.fromLookup(ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName) lookup)
      : _lookup = lookup;

  void store_dart_post_cobject(
    DartPostCObjectFnType ptr,
  ) {
    return _store_dart_post_cobject(
      ptr,
    );
  }

  late final _store_dart_post_cobjectPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(DartPostCObjectFnType)>>('store_dart_post_cobject');
  late final _store_dart_post_cobject = _store_dart_post_cobjectPtr.asFunction<void Function(DartPostCObjectFnType)>();

  void wire_simple_adder_1(
    int port_,
    int a,
    int b,
  ) {
    return _wire_simple_adder_1(
      port_,
      a,
      b,
    );
  }

  late final _wire_simple_adder_1Ptr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Int32, ffi.Int32)>>('wire_simple_adder_1');
  late final _wire_simple_adder_1 = _wire_simple_adder_1Ptr.asFunction<void Function(int, int, int)>();

  void free_WireSyncReturnStruct(
    WireSyncReturnStruct val,
  ) {
    return _free_WireSyncReturnStruct(
      val,
    );
  }

  late final _free_WireSyncReturnStructPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(WireSyncReturnStruct)>>('free_WireSyncReturnStruct');
  late final _free_WireSyncReturnStruct =
      _free_WireSyncReturnStructPtr.asFunction<void Function(WireSyncReturnStruct)>();
}

typedef DartPostCObjectFnType = ffi.Pointer<ffi.NativeFunction<ffi.Bool Function(DartPort, ffi.Pointer<ffi.Void>)>>;
typedef DartPort = ffi.Int64;
