using System;
using System.Runtime.InteropServices;

namespace IronCore
{
    public static class IronCoreNative
    {
        [DllImport("icnative", EntryPoint = "rust_test_function", CallingConvention = CallingConvention.Cdecl)]
        public static extern void RustTestFunction();
    }
}
