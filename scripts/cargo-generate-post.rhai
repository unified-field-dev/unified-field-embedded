// Copy flat host CI into place after Liquid expansion.
system::command("mkdir", ["-p", ".github/workflows"]);
system::command("cp", ["ci/generated-host-ci.yml.example", ".github/workflows/ci.yml"]);
print("Copied .github/workflows/ci.yml from ci/generated-host-ci.yml.example");
