variable "GITHUB_SHA" {
  default = ""
}

variable "VERSION" {
  default = ""
}

target "moose-console" {
    dockerfile = "./apps/moose-console/Dockerfile"
    contexts = {
        app = "./apps/moose-console"
        base = "."
    }
    tags = ["514labs/moose-console:latest", "514labs/moose-console:${GITHUB_SHA}", "514labs/moose-console:${VERSION}"]
}