const __context = global.__modules.register("1000");
const { default: foo } = global.__modules.require("foo");
global.__modules.require("foo");
global.__modules.import("foo");
