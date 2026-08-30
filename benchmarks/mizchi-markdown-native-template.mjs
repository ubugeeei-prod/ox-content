export function renderNativeCommandSource({ nativeLabel, sampleMarkdown, sizeSpecs }) {
  const runSizeCalls = sizeSpecs
    .map(
      ({ name, repeats, iterations }) =>
        `  run_size(${moonbitString(name)}, ${repeats}, ${iterations}, runs)`,
    )
    .join("\n");

  return `let sample_markdown : String = ${moonbitString(sampleMarkdown)}

priv enum Mode {
  Parse
  Render
}

priv struct Measurement {
  ops_per_sec : Double
  avg_ms : Double
  throughput_mbs : Double
}

fn sample_document(repeats : Int) -> String {
  let builder = StringBuilder::new()
  for index in 0..<repeats {
    if index > 0 {
      builder.write_string("\\n\\n")
    }
    builder.write_string(sample_markdown)
  }
  builder.to_string()
}

fn parse_runs(args : Array[String]) -> Result[Int, String] {
  if args.length() != 3 || args[1] != "--runs" {
    return Err("usage: ox-content-competitor --runs <n>")
  }

  let runs = @string.parse_int(args[2]) catch {
    _ => return Err("runs must be a positive integer")
  }
  guard runs > 0 else {
    return Err("runs must be a positive integer")
  }
  Ok(runs)
}

fn run_workload(mode : Mode, input : String) -> Int {
  match mode {
    Parse => @markdown.parse(input).document.children.length()
    Render => @markdown.md_to_html(input).length()
  }
}

fn measure_once(mode : Mode, input : String, iterations : Int) -> Measurement {
  let mut checksum = 0
  for _ in 0..<5 {
    checksum += run_workload(mode, input)
  }

  let start = @bench.monotonic_clock_start()
  for _ in 0..<iterations {
    checksum += run_workload(mode, input)
  }
  let elapsed_us = @bench.monotonic_clock_end(start)
  let avg_ms = elapsed_us / iterations.to_double() / 1000.0
  let ops_per_sec = 1000.0 / avg_ms
  let throughput_mbs = input.length().to_double() / 1024.0 / 1024.0 * ops_per_sec

  if checksum == -1 {
    abort("unreachable")
  }

  {
    ops_per_sec,
    avg_ms,
    throughput_mbs,
  }
}

fn emit_samples(
  suite : String,
  size_name : String,
  input : String,
  iterations : Int,
  mode : Mode,
  runs : Int
) -> Unit {
  for _ in 0..<runs {
    let sample = measure_once(mode, input, iterations)
    println(
      suite +
      "\\t" +
      ${moonbitString(nativeLabel)} +
      "\\t" +
      size_name +
      "\\t" +
      sample.ops_per_sec.to_string() +
      "\\t" +
      sample.avg_ms.to_string() +
      "\\t" +
      sample.throughput_mbs.to_string(),
    )
  }
}

fn run_size(size_name : String, repeats : Int, iterations : Int, runs : Int) -> Unit {
  let input = sample_document(repeats)
  emit_samples("parse", size_name, input, iterations, Mode::Parse, runs)
  emit_samples("render", size_name, input, iterations, Mode::Render, runs)
}

fn run_benchmarks(args : Array[String]) -> Unit {
  let runs = match parse_runs(args) {
    Ok(value) => value
    Err(message) => abort(message)
  }
${runSizeCalls}
}

async fn write_error(message : String, exit_code : Int) -> Unit {
  @stdio.stderr.write(message + "\\n") catch {
    _ => ()
  }
  @sys.exit(exit_code)
}

async fn render_stdin() -> Unit {
  let source = @stdio.stdin.read_all().text() catch {
    error => {
      write_error("failed to read stdin: \\{error}", 2)
      return
    }
  }

  @stdio.stdout.write(@markdown.md_to_html(source, autolink=false, tagfilter=false)) catch {
    error => write_error("failed to write stdout: \\{error}", 2)
  }
}

async fn main {
  let args = @env.args()
  if args.length() == 2 && args[1] == "--render-stdin" {
    render_stdin()
    return
  }
  run_benchmarks(args)
}
`;
}

function moonbitString(value) {
  return `"${value
    .replaceAll("\\", "\\\\")
    .replaceAll('"', '\\"')
    .replaceAll("\r", "\\r")
    .replaceAll("\n", "\\n")
    .replaceAll("\t", "\\t")}"`;
}
