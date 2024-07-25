# Terustry
Simple configurable proxy that implement [terraform provider registry protocol](https://www.terraform.io/docs/internals/provider-registry-protocol.html), to build your own terraform provider private registry.

### How it works
Terustry use a yaml file to describe how to discover versions and download urls.
```yaml
providers:
  - name: hashicorp/hashicups # namespace/name of your provider
    protocols: [5.0]
    version: # vcs to fetch provider versions (gitlab and github are supported)
      type: github 
      uri: https://api.github.com/repos/hashicorp/terraform-provider-hashicups/releases # url of the release api of your vcs
      token: "{{terustry_github_token}}"
    binaries: [{os: linux, arch: arm64}]
    signature: # information about key used to sign your provider
      key_id: 97751AE79C450B19
      key_armor: "-----BEGIN PGP PUBLIC KEY BLOCK-----"
    artifact: # describe how to build download urls
      filename: terraform-provider-hashicups_{{version}}_{{os}}_{{arch}}.zip
      download_url: https://.../v{{version}}/terraform-provider-hashicups_{{version}}_{{os}}_{{arch}}.zip
      shasums_url: https://.../v{{version}}/terraform-provider-hashicups_{{version}}_SHA256SUMS
      shasums_signature_url: https://.../v{{version}}/terraform-provider-hashicups_{{version}}_SHA256SUMS.sig
```

Terustry will parse the result of the release api you provide (`version.uri`), assuming each release published is a provider version.

Then it will use the `artifact` section to build the download urls of your provider.


### Run

#### With docker
```bash
docker run -p 8080:8080 -e TERUSTRY_GITHUB_TOKEN='XXX' -v $(pwd)/terustry-sample-github.yml:/etc/terustry.yml --rm -it vptech/terustry
```

#### With docker build
```bash
docker build -t terustry .
docker run -p 8080:8080 -e TERUSTRY_GITHUB_TOKEN='XXX' -v $(pwd)/terustry-sample-github.yml:/etc/terustry.yml --rm -it terustry
```
#### With cargo
```bash
TERUSTRY_GITHUB_TOKEN=XXXX cargo run -- --config terustry-sample-github.yml
```

If you want to embed the configuration in docker image, juste create a `terustry.yml` file with your configuration.

### Test
#### With curl
```bash
$ curl http://localhost:8080/terraform/providers/v1/hashicorp/hashicups/versions
```
```javascript
{
  id: "hashicorp/hashicups",
  versions: [{
    version: "0.3.1",
    protocols: [
      "5.0"
    ],
    platforms: [{
      os: "freebsd",
      arch: "386"
    }
  ]}]
}
```
#### With terraform
```terraform
terraform {
  required_providers {
    hashicups = {
      source = "localhost:8081/hashicorp/hashicups"
      version = "0.3.1"
    }
  }
}

provider "hashicups" {
  # Configuration options
}
```

```bash
$ terraform init
```
##### Local ssl
Terraform provider registry need to have a valid SSL certificate to work.

If you want to test the all thing (`terraform init`) locally, you have to have a "ssl proxy".

Install [mkcert](https://github.com/FiloSottile/mkcert) and [local-ssl-proxy](https://github.com/cameronhunter/local-ssl-proxy)

```bash
mkcert install
mkcert localhost
local-ssl-proxy --source 8081 --target 8080 --key localhost-key.pem --cert localhost.pem
```

### Caching

By default, Terustry will cache responses from Github/Gitlab for 10 minutes. This
may result in an unwanted behaviour where a recently released version for a given
provider is not available.

The new version will become available once the cache is refreshed.

However, if you need a faster refresh timing, for example in a CI/CD pipeline, you
may request a specific cache entry to be invalidated using the following route:
`GET /terraform/providers/v1/{namespace}/{provider_name}/invalidate`

This should result in an empty 200 OK response.

For example:
```bash
curl http://localhost:8080/terraform/providers/v1/hashicorp/hashicups/invalidate
```