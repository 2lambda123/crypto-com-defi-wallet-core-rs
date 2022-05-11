# This file has been generated by node2nix 1.11.0. Do not edit!

{ nodeEnv, fetchurl, fetchgit, nix-gitignore, stdenv, lib, globalBuildInputs ? [ ] }:

let
  sources = {
    "bash-color-0.0.4" = {
      name = "bash-color";
      packageName = "bash-color";
      version = "0.0.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/bash-color/-/bash-color-0.0.4.tgz";
        sha1 = "e9be8ce33540cada4881768c59bd63865736e913";
      };
    };
    "commander-2.11.0" = {
      name = "commander";
      packageName = "commander";
      version = "2.11.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/commander/-/commander-2.11.0.tgz";
        sha512 = "b0553uYA5YAEGgyYIGYROzKQ7X5RAqedkfjiZxwi0kL1g3bOaBNNZfYkzt/CL0umgD5wc9Jec2FbB98CjkMRvQ==";
      };
    };
    "fs-extra-3.0.1" = {
      name = "fs-extra";
      packageName = "fs-extra";
      version = "3.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/fs-extra/-/fs-extra-3.0.1.tgz";
        sha1 = "3794f378c58b342ea7dbbb23095109c4b3b62291";
      };
    };
    "graceful-fs-4.2.0" = {
      name = "graceful-fs";
      packageName = "graceful-fs";
      version = "4.2.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/graceful-fs/-/graceful-fs-4.2.0.tgz";
        sha512 = "sha512-jpSvDPV4Cq/bgtpndIWbI5hmYxhQGHPC4d4cqBPb4DLniCfhJokdXhwhaDuLBGLQdvvRum/UiX6ECVIPvDXqdg==";
      };
    };
    "jsonfile-3.0.1" = {
      name = "jsonfile";
      packageName = "jsonfile";
      version = "3.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/jsonfile/-/jsonfile-3.0.1.tgz";
        sha1 = "a5ecc6f65f53f662c4415c7675a0331d0992ec66";
      };
    };
    "lodash-4.17.4" = {
      name = "lodash";
      packageName = "lodash";
      version = "4.17.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/lodash/-/lodash-4.17.4.tgz";
        sha1 = "78203a4d1c328ae1d86dca6460e369b57f4055ae";
      };
    };
    "minimist-0.0.10" = {
      name = "minimist";
      packageName = "minimist";
      version = "0.0.10";
      src = fetchurl {
        url = "https://registry.npmjs.org/minimist/-/minimist-0.0.10.tgz";
        sha1 = "de3f98543dbf96082be48ad1a0c7cda836301dcf";
      };
    };
    "npm-2.15.12" = {
      name = "npm";
      packageName = "npm";
      version = "2.15.12";
      src = fetchurl {
        url = "https://registry.npmjs.org/npm/-/npm-2.15.12.tgz";
        sha1 = "df7c3ed5a277c3f9d4b5d819b05311d10a200ae6";
      };
    };
    "npm-5.1.0" = {
      name = "npm";
      packageName = "npm";
      version = "5.1.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/npm/-/npm-5.1.0.tgz";
        sha512 = "pt5ClxEmY/dLpb60SmGQQBKi3nB6Ljx1FXmpoCUdAULlGqGVn2uCyXxPCWFbcuHGthT7qGiaGa1wOfs/UjGYMw==";
      };
    };
    "npmi-1.0.1" = {
      name = "npmi";
      packageName = "npmi";
      version = "1.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/npmi/-/npmi-1.0.1.tgz";
        sha1 = "15d769273547545e6809dcf0ce18aed48b0290e2";
      };
    };
    "optimist-0.6.1" = {
      name = "optimist";
      packageName = "optimist";
      version = "0.6.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/optimist/-/optimist-0.6.1.tgz";
        sha1 = "da3ea74686fa21a19a111c326e90eb15a0196686";
      };
    };
    "os-homedir-1.0.2" = {
      name = "os-homedir";
      packageName = "os-homedir";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/os-homedir/-/os-homedir-1.0.2.tgz";
        sha1 = "ffbc4988336e0e833de0c168c7ef152121aa7fb3";
      };
    };
    "os-tmpdir-1.0.2" = {
      name = "os-tmpdir";
      packageName = "os-tmpdir";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/os-tmpdir/-/os-tmpdir-1.0.2.tgz";
        sha1 = "bbe67406c79aa85c5cfec766fe5734555dfa1274";
      };
    };
    "q-1.5.0" = {
      name = "q";
      packageName = "q";
      version = "1.5.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/q/-/q-1.5.0.tgz";
        sha1 = "dd01bac9d06d30e6f219aecb8253ee9ebdc308f1";
      };
    };
    "semver-4.3.6" = {
      name = "semver";
      packageName = "semver";
      version = "4.3.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/semver/-/semver-4.3.6.tgz";
        sha1 = "300bc6e0e86374f7ba61068b5b1ecd57fc6532da";
      };
    };
    "semver-5.3.0" = {
      name = "semver";
      packageName = "semver";
      version = "5.3.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/semver/-/semver-5.3.0.tgz";
        sha1 = "9b2ce5d3de02d17c6012ad326aa6b4d0cf54f94f";
      };
    };
    "tmp-0.0.31" = {
      name = "tmp";
      packageName = "tmp";
      version = "0.0.31";
      src = fetchurl {
        url = "https://registry.npmjs.org/tmp/-/tmp-0.0.31.tgz";
        sha1 = "8f38ab9438e17315e5dbd8b3657e8bfb277ae4a7";
      };
    };
    "universalify-0.1.2" = {
      name = "universalify";
      packageName = "universalify";
      version = "0.1.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/universalify/-/universalify-0.1.2.tgz";
        sha512 = "rBJeI5CXAlmy1pV+617WB9J63U6XcazHHF2f2dbJix4XzpUF0RS3Zbj0FGIOCAva5P/d/GBOYaACQ1w+0azUkg==";
      };
    };
    "user-home-2.0.0" = {
      name = "user-home";
      packageName = "user-home";
      version = "2.0.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/user-home/-/user-home-2.0.0.tgz";
        sha1 = "9c70bfd8169bc1dcbf48604e0f04b8b49cde9e9f";
      };
    };
    "wordwrap-0.0.3" = {
      name = "wordwrap";
      packageName = "wordwrap";
      version = "0.0.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/wordwrap/-/wordwrap-0.0.3.tgz";
        sha1 = "a3d5da6cd5c0bc0008d37234bbaf1bed63059107";
      };
    };
  };
in
{
  "gitbook-cli-2.3.2" = nodeEnv.buildNodePackage {
    name = "gitbook-cli";
    packageName = "gitbook-cli";
    version = "2.3.2";
    src = fetchurl {
      url = "https://registry.npmjs.org/gitbook-cli/-/gitbook-cli-2.3.2.tgz";
      sha512 = "eyGtkY7jKHhmgpfuvgAP5fZcUob/FBz4Ld0aLRdEmiTrS1RklimN9epzPp75dd4MWpGhYvSbiwxnpyLiv1wh6A==";
    };
    dependencies = [
      sources."bash-color-0.0.4"
      sources."commander-2.11.0"
      sources."fs-extra-3.0.1"
      sources."graceful-fs-4.2.0"
      sources."jsonfile-3.0.1"
      sources."lodash-4.17.4"
      sources."minimist-0.0.10"
      sources."npm-5.1.0"
      (sources."npmi-1.0.1" // {
        dependencies = [
          sources."npm-2.15.12"
          sources."semver-4.3.6"
        ];
      })
      sources."optimist-0.6.1"
      sources."os-homedir-1.0.2"
      sources."os-tmpdir-1.0.2"
      sources."q-1.5.0"
      sources."semver-5.3.0"
      sources."tmp-0.0.31"
      sources."universalify-0.1.2"
      sources."user-home-2.0.0"
      sources."wordwrap-0.0.3"
    ];
    buildInputs = globalBuildInputs;
    meta = {
      description = "CLI to generate books and documentation using gitbook";
      homepage = "https://www.gitbook.com";
      license = "Apache-2.0";
    };
    production = true;
    bypassCache = true;
    reconstructLock = true;
  };
}
