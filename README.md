# copr resource for concourse

A resource intended to make use of the copr infrastructure

https://copr.fedoraproject.org/

http://copr-rest-api.readthedocs.io/en/latest/index.html


## usage

	resource_types:
	- name: copr
	  type: docker-image
	  source:
	    repository: quay.io/ahoi/concourse-copr-resource

	resources:
	- name: copr-resource
	  type: copr
	  source:
	    project_id: {{copr-project-id}}
	    login: {{copr-login}}
	    token: {{copr-token}}
	    url: {{copr-url}}
	    regex: ".*\\.src\\.rpm"

## example

See [oregano](https://github.com/drahnr/oregano) for up to date usage.

### source

* `login` copr login as provided by the API page
* `token` copr token used to login in combination with `login`
* `url` copr url, usually `https://copr.fedorainfracloud.org/api_2/builds` or whatever you see while logged in with [copr](https://copr.fedoraproject.org/api/)

### check

always returns `[]` which means no new version, not intended to be checked for

### in

`nop`, not intended to be pulled in

### out

Pushes a local srpm to copr

* `rpmbuild_dir` the path to the `rpmbuild` base directory which is expected to contain the srpm somewhere underneath
* `project_id` the associated project_id `curl -u "$LOGIN:$TOKEN" -X GET https://copr.fedorainfracloud.org/api_2/projects?name=$PROJECT_NAME | jq -r ".projects[].project.id"` to
* `regex` regular expression to match the path to the srpm including the srpm name, capture the name as group if possible, if multiple match, the first one is choosen - the default should be fine
* `chroots` list of change roots, default: `["fedora-25-x86_64"]`
* `enable_net` : enable only if you need the web for building the rpm from your srpm, default: `false`
* `max_n_bytes` : the maximum total number of bytes to push, default: `1000000000`

