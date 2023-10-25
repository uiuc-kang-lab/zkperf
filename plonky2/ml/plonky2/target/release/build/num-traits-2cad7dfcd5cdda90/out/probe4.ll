; ModuleID = 'probe4.ccbd5c6a8a6bc402-cgu.0'
source_filename = "probe4.ccbd5c6a8a6bc402-cgu.0"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx11.0.0"

@alloc_ff34217a6603ee9aaa37f878633082dd = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/1c05d50c8403c56d9a8b6fb871f15aaa26fb5d07/library/core/src/num/mod.rs" }>, align 1
@alloc_575b46658ee33407357120142bd2e06a = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_ff34217a6603ee9aaa37f878633082dd, [16 x i8] c"K\00\00\00\00\00\00\00v\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe4::probe
; Function Attrs: uwtable
define void @_ZN6probe45probe17hdc05faa8e883e116E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h608a3d658213fbcaE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h3d95c27cca824567E(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_575b46658ee33407357120142bd2e06a) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h608a3d658213fbcaE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17h3d95c27cca824567E(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.75.0-nightly (1c05d50c8 2023-10-21)"}
