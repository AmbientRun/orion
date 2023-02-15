
import { init, WASI } from '@wasmer/wasi';
import wasmModule from "./orion-client.wasm"

export async function run() {

  // This is needed to load the WASI library first (since is a Wasm module)
  await init();
  console.log("Creating wasi instance")

  let wasi = new WASI({
    env: {
      // 'ENVVAR1': '1',
      // 'ENVVAR2': '2'
    },
    args: [
      // 'command', 'arg1', 'arg2'
    ],
  });

  const moduleBytes = fetch(wasmModule);
  const module = await WebAssembly.compileStreaming(moduleBytes);
  // Instantiate the WASI module
  await wasi.instantiate(module, {});

  // Run the start function
  let exitCode
  try {
    exitCode = wasi.start();

  } catch (error) {
    console.log(`Failed run wasi module:\n${error}`)
  }
  let stdout = wasi.getStdoutString();
  let stderr = wasi.getStderrString();

  // This should print "hello world (exit code: 0)"
  // console.log(`${stdout(exit code: ${ exitCode })`);
  console.log({ stdout, stderr, exitCode })
  // let args = ["bin", "arg1", "arg2"];
  // let env = ["FOO=bar"];
  // let fds = [
  //   // new File([]), // stdin
  //   // new File([]), // stdout
  //   // new File([]), // stderr
  //   // new PreopenDirectory(".", {
  //   //   "example.c": new File(new TextEncoder("utf-8").encode(`#include "a"`)),
  //   //   "hello.rs": new File(new TextEncoder("utf-8").encode(`fn main() { println!("Hello World!"); }`)),
  //   // }),
  // ];
  // let wasi = new WASI(args, env, fds);

  // let wasm = await WebAssembly.compileStreaming(fetch("bin.wasm"));
  // console.log({ wasm })
  // let inst = await WebAssembly.instantiate(wasm, {
  //   "wasi_snapshot_preview1": wasi.wasiImport,
  // });
  // wasi.start(inst);
}
