global.__modules.define(function(__context) {
  const { default: React, useState, useCallback } = __context.require("1000", 0);
  const { foo } = __context.require("1001", 1);
  const { bar: bar2 } = __context.require("1002", 2);
  const { baz } = __context.require("1003", 3);
  const { foo2 } = __context.require("1001", 4);
  const __mod = __context.require("1009", 7);
  const __mod1 = __context.require("1011", 8);
  const __mod2 = __context.require("1012", 9);
  const __mod3 = __context.require("1013", 10);
  const __mod4 = __context.require("1014", 11);
  React.lazy(()=>__context.import("1004", 5));
  if (__DEV__) {
      __context.require("1005", 6);
  }
  const value = 'val';
  module.exports = __context.module.exports = 'cjs';
  module.exports.foo = __context.module.exports.foo = 2;
  Object.assign(module.exports = __context.module.exports, {
      bar: 1
  });
  const variable = 1;
  class Class {
  }
  function func() {}
  function __default() {
      __context.require("1006", 7);
      const inner = async ()=>{
          await __context.import("1008", 8);
          __context.require("1007", 9);
      };
  }
  __x = variable, __x1 = Class, __x2 = func, __x3 = __default, __x4 = value, __x5 = foo, __x6 = foo2, __x7 = baz, __x8 = baz;
  __context.exports(function() {
      return {
          "variable": __x,
          "Class": __x1,
          "func": __x2,
          "default": __x3,
          "value2": __x4,
          "foo": __x5,
          "foo2": __x6,
          "baz": __x7,
          "baz2": __x8,
          ...__context.exports.ns(__mod),
          "rx": __context.exports.ns(__mod1),
          rx0: __mod2.rx0,
          rx1: __mod2.rx1,
          rx2: __mod2.rx2,
          rx4: __mod3.rx3,
          rx5: __mod4.default
      };
  });
}, "1000");
var __x, __x1, __x2, __x3, __x4, __x5, __x6, __x7, __x8;
