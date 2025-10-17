## CH3

### QUIZ

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2025S/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

   rbi 版本：默认 [rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

   * ch2b_bad_address: 程序想去访问0x0，但是触发了pagefault，被kernel结束掉了
   * ch2b_bad_instructions程序想去执行sret指令，这个指令是S模式才能执行的，所以触发了trap，被kernel结束了
   * ch2b_bad_register程序想去读取sstatus寄存器，但是缺少权限，无法在U模式下做这个操作，所以触发了Illegal Instruction，被kernel结束了

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2025S/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

   1. L40：刚进入 `__restore` 时，`sp` 代表了什么值。请指出 `__restore` 的两种使用情景。

      刚进入的时候sp指向的是kernel stack，可以从sp处拿到传参

   2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

      ```
      ld t0, 32*8(sp)
      ld t1, 33*8(sp)
      ld t2, 2*8(sp)
      csrw sstatus, t0
      csrw sepc, t1
      csrw sscratch, t2
      ```

      | CSR 名   | 相关的功能                                                   |
      | -------- | ------------------------------------------------------------ |
      | sstatus  | `SPP` 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息 |
      | sepc     | 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址 |
      | sscratch | 在用户态，`sscratch` 保存内核栈的地址；在内核态，`sscratch` 的值为 0。 |

   3. L50-L56：为何跳过了 `x2` 和 `x4`？

      ```
      ld x1, 1*8(sp)
      ld x3, 3*8(sp)
      .set n, 5
      .rept 27
         LOAD_GP %n
         .set n, n+1
      .endr
      ```

      x4是线程指针我们在这里没有用到所以不需要处理。

      x2是栈指针，返回时我们已经在最后处理了sp了（读取2*8(sp)的值到t2，然后从t2赋值到sscratch，再交换进sp），所以就没有必要再处理x2

   4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      ```
      csrrw sp, sscratch, sp
      ```

      在执行这条指令之前，sp指向kernel stack，sscratch是user stack，执行时候，sp和sscratch的值会发生交换，导致sp指向user stack，sscratch指向kernel stack。这样子可以在程序返回userspace的时候继续指向正确的栈帧位置

   5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

      `sret`，该指令就是从S态返回到U态，表明一个trap handling的结束

   6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      ```
      csrrw sp, sscratch, sp
      ```

      跟（4）相反，sscratch在这条指令之前存储的是S态的栈，sp指向U态的栈；执行之后sp指向kernel的栈，sscratch指向user的栈

   7. 从 U 态进入 S 态是哪一条指令发生的？

      `ecall`，该指令会trap到S态

### EXP

#### 要求

在 ch3 中，我们的系统已经能够支持多个任务分时轮流运行，我们希望引入一个新的系统调用 [``](https://learningos.cn/rCore-Tutorial-Guide-2025S/chapter3/5exercise.html#id3)sys_trace``（ID 为 410）用来追踪当前任务系统调用的历史信息，并做对应的修改。定义如下。

```
fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize
```

- 调用规范：

  这个系统调用有三种功能，根据 `trace_request` 的值不同，执行不同的操作：如果 `trace_request` 为 0，则 `id` 应被视作 `*const u8` ，表示读取当前任务 `id` 地址处一个字节的无符号整数值。此时应忽略 `data` 参数。返回值为 `id` 地址处的值。如果 `trace_request` 为 1，则 `id` 应被视作 `*mut u8` ，表示写入 `data` （作为 `u8`，即只考虑最低位的一个字节）到该用户程序 `id` 地址处。返回值应为0。如果 `trace_request` 为 2，表示查询当前任务调用编号为 `id` 的系统调用的次数，返回值为这个调用次数。**本次调用也计入统计** 。否则，忽略其他参数，返回值为 -1。

- 说明：

  你可能会注意到，这个调用的读写并不安全，使用不当可能导致崩溃。这是因为在下一章节实现地址空间之前，系统中缺乏隔离机制。所以我们 **不要求你实现安全检查机制，只需通过测试用例即可** 。你还可能注意到，这个系统调用读写本任务内存的功能并不是很有用。这是因为作业的灵感来源 syscall 主要依靠 trace 功能追踪其他任务的信息，但在本章节我们还没有进程、线程等概念，所以简化了操作，只要求追踪自身的信息。

#### 实现

前两个功能（读出和存入*_id*处对应的u8字节）较为简单，我们几行代码搞定：

```rust
pub unsafe fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            // read *_id as isize
            *(_id as *const u8) as isize
        },
        1 => {
            // *_id = _data
            (_id as *mut u8).write_volatile(_data as u8);
            0
        },
        _ => panic!("Unsupported trace request!"),
    }
}
```

考虑到这些都是unsafe，于是我干脆把函数也声明成为了unsafe，免得再在函数里面写`unsafe{}`的代码块

第三个syscall的trace，考虑到其通用性，把它当作task的子功能并不能完全满足期望。

我期望的是它可以满足以下几个需求：

* 每个app都有自己单独的trace数据
* 每个app的trace数据跟app本身的生命周期相绑定
* syscall的计数要占用最少的空间
  * 例如使用了某个syscall，才会创建syscall的表项给这个syscall去计数

但是在CH3阶段下，后两个都无法实现，因为其要求kernel本身要有内存管理，也就是说在CH3里，我们只能操纵定长的数据，无法动态分配任何东西，所以Rust的智能指针`Box`和一些便捷的容器例如`vector`、`map`都无法使用。

考虑到以后拓展的需求，即kernel实现了动态内存管理的情况下，我们能干的事情更多，所以我们当次只实现一个最基本的module，它使用定长的二维数组记录每个app发动的每个syscall的数量：

```rust
static mut __TRACE: [[usize; 6]; MAX_APP_NUM] =  [[0 as usize; 6]; MAX_APP_NUM];

fn get_internal_id_from_syscall(syscall: usize) -> usize {
    match syscall{
        SYSCALL_WRITE => 0,
        SYSCALL_EXIT => 1,
        SYSCALL_YIELD => 2,
        SYSCALL_GET_TIME => 3,
        SYSCALL_TRACE => 4,
        _=> {println!("unsupported syscall {}, assigning as 6", syscall); 6 }
    }
}
```

把trace模块实现成module还有一个好处就是让它和其他模块解耦，例如task模块就专注于任务的载入、启动、切换，不应该去记录任务的数据。这也是参考了linux kernel tracer的实现方式。

我们将已有的五种syscall还有其余尚未支持的映射到0-5，然后再在task模块里新增一个调用可以得到当前的task id，这样我们就可以唯一地从task id和syscall id映射到syscall trace表项：

```rust
pub fn trace_syscall(syscall: usize) -> () {
    let id = get_current_task_id();
    let trace_id = get_internal_id_from_syscall(syscall);
    unsafe {__TRACE[id][trace_id] += 1;}
}

pub unsafe fn get_syscall(syscall: usize) -> usize {
    let id = get_current_task_id();
    let trace_id = get_internal_id_from_syscall(syscall);
    unsafe {__TRACE[id][trace_id]}
}
```

操作全局static数组所使用的static是必不可少的。然后将这两个函数放到正确的位置即可，具体实现可见https://github.com/LearningOS/2025a-rcore-ukouji/tree/ch3

#### 改进空间

当前的实现已经满足了课设要求，不过确实有很多可以改进的空间，有句话说过早的改进是万恶之源，我们可以在之后kernel的功能愈发丰富之后再考虑改进或重构：

* 对__TRACE数组的unsafe读写可以重构为带互斥锁的读写以保证安全性，或者模仿TASK_MANAGER一样做个UPCELL使得race condition发生时kernel panic
* 使用动态内存分配机制而非指定全局静态数组