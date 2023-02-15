// import "orion-client";
import { WASI, File, PreopenDirectory } from "@bjorn3/browser_wasi_shim";
import { Fd } from "./node_modules/@bjorn3/browser_wasi_shim/src/fd";

import wasmModule from "./orion-client.wasm"

export async function run() {

  let args = [];
  let env = [];

  class OnOutput extends Fd {
    constructor(func) {
      super();
      this.decoder = new TextDecoder()
      this.func = func
    }
    fd_write(view8/*: Uint8Array*/, iovs/*: [wasi.Iovec]*/)/*: {ret: number, nwritten: number}*/ {
      let nwritten = 0;
      let str = ""
      for (let iovec of iovs) {
        // console.log(iovec.buf_len, iovec.buf_len, view8.slice(iovec.buf, iovec.buf + iovec.buf_len));
        let buffer = view8.slice(iovec.buf, iovec.buf + iovec.buf_len);
        str += this.decoder.decode(buffer)
        nwritten += iovec.buf_len;
      }
      this.func(str)
      return { ret: 0, nwritten };
    }
  }

  let fds = [
    new File([]), // stdin
    new OnOutput(s => console.log(s)), // stdout
    new OnOutput(s => console.log(s)), // stderr
  ];
  let wasi = new WASI(args, env, fds);

  /// Compile the module
  let wasm = await WebAssembly.compileStreaming(fetch(wasmModule));
  console.log("Compiled wasm module")
  /// Instantiate the wasm module into a stateful executable
  let instance = await WebAssembly.instantiate(wasm, {
    "wasi_snapshot_preview1": wasi.wasiImport,
  });

  console.log("Instantiated wasm module")
  console.log({ exports: instance.exports });
  // let exitCode = wasi.start(inst)

  wasi.inst = instance;
  let ret = instance.exports.run();

  console.log(`Returned: ${ret}`);

  // console.log("Initializing wasm")

  // // This is needed to load the WASI library first (since is a Wasm module)
  // await init();
  // console.log("Creating wasi instance")

  // let wasi = new WASI({
  //   env: {
  //     // 'ENVVAR1': '1',
  //     // 'ENVVAR2': '2'
  //   },
  //   args: [
  //     // 'command', 'arg1', 'arg2'
  //   ],
  // });

  // const moduleBytes = fetch(wasm_module);
  // const module = await WebAssembly.compileStreaming(moduleBytes);
  // // Instantiate the WASI module
  // await wasi.instantiate(module, {});

  // // Run the start function
  // let exitCode
  // try {
  //   exitCode = wasi.start();

  // } catch (error) {
  //   console.log(`Failed run wasi module:\n${error}`)
  // }
  // let stdout = wasi.getStdoutString();
  // let stderr = wasi.getStderrString();

  // // This should print "hello world (exit code: 0)"
  // // console.log(`${stdout(exit code: ${ exitCode })`);
  // console.log({ stdout, stderr, exitCode })
  // // let args = ["bin", "arg1", "arg2"];
  // // let env = ["FOO=bar"];
  // // let fds = [
  // //   // new File([]), // stdin
  // //   // new File([]), // stdout
  // //   // new File([]), // stderr
  // //   // new PreopenDirectory(".", {
  // //   //   "example.c": new File(new TextEncoder("utf-8").encode(`#include "a"`)),
  // //   //   "hello.rs": new File(new TextEncoder("utf-8").encode(`fn main() { println!("Hello World!"); }`)),
  // //   // }),
  // // ];
  // // let wasi = new WASI(args, env, fds);

  // // let wasm = await WebAssembly.compileStreaming(fetch("bin.wasm"));
  // // console.log({ wasm })
  // // let inst = await WebAssembly.instantiate(wasm, {
  // //   "wasi_snapshot_preview1": wasi.wasiImport,
  // // });
  // // wasi.start(inst);
}

// let canvas = document.getElementById("canvas")



// let game = wasm.Game.new()
// console.log(`Created new game: ${ game }`)

// canvas.width = document.body.clientWidth;
// canvas.height = document.body.clientHeight;

// let prev_time = performance.now()
// const renderLoop = (now) => {
//   let dt = (now - prev_time) / 1000;
//   prev_time = now;
//   game.update(dt);

//   // Clear
//   const context = canvas.getContext('2d');
//   context.clearRect(0, 0, canvas.width, canvas.height);

//   game.render(canvas)

//   requestAnimationFrame(renderLoop);
// };

// requestAnimationFrame(renderLoop);
