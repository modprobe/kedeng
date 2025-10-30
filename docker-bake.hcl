group "default" {
    targets = ["receiver", "data_importer", "persister", "api"]
}

target "receiver" {
    context = "./packages/receiver"
    dockerfile = "Dockerfile"
    tags = ["ghcr.io/modprobe/kedeng-receiver:latest"]
    labels = {
        "org.opencontainers.image.source" = "https://github.com/modprobe/kedeng"
    }
    platforms = [ "linux/amd64" ]
}

target "data_importer" {
    context = "./packages/data-importer"
    dockerfile = "Dockerfile"
    tags = ["ghcr.io/modprobe/kedeng-data-importer:latest"]
    labels = {
        "org.opencontainers.image.source" = "https://github.com/modprobe/kedeng"
    }
    platforms = [ "linux/amd64" ]
}

target "persister" {
    context = "."
    dockerfile = "./packages/persister/Dockerfile"
    tags = ["ghcr.io/modprobe/kedeng-persister:latest"]
    labels = {
        "org.opencontainers.image.source" = "https://github.com/modprobe/kedeng"
    }
    platforms = [ "linux/amd64" ]
}

target "api" {
    context = "./packages/api"
    dockerfile = "Dockerfile"
    tags = [ "ghcr.io/modprobe/kedeng-api" ]
    labels = {
        "org.opencontainers.image.source" = "https://github.com/modprobe/kedeng"
    }
    platforms = [ "linux/amd64" ]
}