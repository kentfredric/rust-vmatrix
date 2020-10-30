<html>
  <head>
    <title>RustC build testing of crates</title>
    <style>
      .hint {
        color: #00DDFF;
      }
      a {
        color: cornflowerblue;
        text-decoration: none;
      }
      a:hover,a:focus,a:active {
        text-decoration: solid coral underline;
      }
      a:visited {
        color: chocolate;
      }
      h1 > a, h2 > a {
        font-size: 50%;
        color: grey;
        text-decoration: none;
      }
      span.version{
        display: inline-block;
        min-width: 5em;
      }
      span.grade {
        margin-left: 3px;
        font-size: 90%;
        display: inline-block;
        min-with: 5em;
      }
      span.numeric_1, span.numeric_2, span.numeric_3, span.numeric_4, span.numeric_5, span.numeric_6,
      span.numeric_7, span.numeric_8, span.numeric_9
      {
        height: 12px;
        border-radius: 5px;
      }
      span.msrv {
        font-size: 80%;
        font-family: monospace;
        display: inline-block;
        color: grey;
        margin-left: 10px;
        margin-right: 10px;
        min-width: 5em;
      }
      span.unknown_msrv { color: hsl(0, 95%, 60% ); font-weight: 900; }
      span.max_msrv { color: hsl(30, 90%, 60% ); font-weight: 900; }
/*
   Width multiplies by 1.5 with int rounding,
   hue goes up 20 per point, saturation goes up 3% per point, and lightness
   goes down 2% per point, giving a shade spectrum from soft light blue to an intense
   red with severity */
      span.numeric_1 { width: 4px;   background-color: hsl(200,70%,76%); }
      span.numeric_2 { width: 6px;   background-color: hsl(220,73%,74%); }
      span.numeric_3 { width: 9px;   background-color: hsl(240,76%,72%); }
      span.numeric_4 { width: 13px;  background-color: hsl(260,79%,70%); }
      span.numeric_5 { width: 20px;  background-color: hsl(280,82%,68%); }
      span.numeric_6 { width: 30px;  background-color: hsl(300,85%,66%); }
      span.numeric_7 { width: 45px;  background-color: hsl(320,88%,64%); }
      span.numeric_8 { width: 68px;  background-color: hsl(340,91%,62%); }
      span.numeric_9 { width: 102px; background-color: hsl(360,94%,60%); }
    </style>
  </head>
  <body>
    <p>[ <a href="#about">about</a> | <a href="#methodology">methodology</a> ]</p>
    <h1 id="reports">Build reports<a href="#reports">#</a></h1>
    <!-- build reports -->
    <h1 id="about">About<a href="#about">#</a></h1>
    <p>This is a tiny<span class="hint" title="lol, well, not in terms of resources needed">*</span>
       project to map what you get when you try to compile given
       versions of given crates against given rustc versions.</p>
    <p>The Objective is to properly map out some sense of
       <em>practical</em> <strong class="hint" title="Minimal Supported Rust Version">MSRV</strong>
       to consider when trying to yourself declare a <strong>MSRV</strong>,
       and pick dependencies based on <strong>MSRV</strong> support,
       and/or, trying to find version combinations that "<em>work</em>"
       when building an arbitrary crate on an arbitray rust
       target.</p>
    <p>It is in <em>no</em> way comprehensive, and is just a
       <em>minimal best effort</em> strategy to narrow down your
       problem quickly.</p>
    <h1 id="methodology">Methodology<a href="#methodolgy">#</a></h1>
    <p>The approach I've used is so simple, it could be called
       "<em>rudimentary</em>".</p>
    <p>For each <code>rustc</code>, and for each <code>crate</code>
       version, construct a dummy project layout that depends on an
       exact version of that, and see if it compiles</p>
    <code><pre># $tmpdir/Cargo.toml
[package]
name = "test"
version = "0.1.0"
authors = ["Kent Fredric &lt;kentnl@gentoo.org&gt;"]

[dependencies.CRATENAME]
version = "=VERSION"
# $tmpdir/src/lib.rs
# *empty file*
</pre></code>
    <p><strong>Note</strong> that it doesn't even think about
       features, and if there are second-order dependencies that
       just <strong>happen</strong> to get pulled in, and
       <em>happen</em> to themselves be broken, then this scenario
       reports a <strong>false negative</strong>.</p>
    <p>It also reports <strong>false negatives</strong> when
       dependencies don't strictly follow semver, and break API,
       leading to compile failures (and also when dependencies are
       not sufficiently semver-binding).</p>
    <p>It is also somewhat subject to <strong>passes</strong> being
       eventually downgraded to <strong>failures</strong>, if the
       version of the dependency used in the test worked at the time,
       but a future version breaks either <strong>MSRV</strong> or
       <strong>API</strong> and otherwise circumvents semver
       bindings.</p>
    <p>It is in some casess possible to work around these situations
       through careful package selection, and some data about that
       somewhere is also on a whishlist for when I have some clue how
       to do such a thing.</p>
    <p><strong>Notedly</strong>, these risks are why I've prioritized
       getting coverage of <em>fundamental</em> crates, particularly,
       ones with either few, or no dependencies, or at least,
       recursively checking the dependencies themselves.</p>
  </body>
</html>
