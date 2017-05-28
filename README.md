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
	    project_id: 825
	    login: {{copr-login}}
	    username: {{copr-username}}
	    token: {{copr-token}}
	    url: {{copr-url}}
	    regex: ".*/oregano.*\\.src\\.rpm"

## example

See [oregano](https://github.com/drahnr/oregano) for up to date usage.

### source

* `username` copr username you use to log in on the web interface
* `login` copr login
* `token` copr token used to login in combination with `login`
* `url` copr url, usually https://copr.fedorainfracloud.org or whatever you see while logged in at https://copr.fedoraproject.org/api/

### check

always returns `[]` which means no new version, not intended to be checked for

### in

`unimpletented!()`, not intended to be pulled in

### out

Pushes a local srpm to copr

* `project_id` the associated project_id `curl -u "$LOGIN:$TOKEN" -X GET https://copr.fedorainfracloud.org/api_2/projects?name=$PROJECT_NAME | jq -r ".projects[].project.id"` to
* `regex` regular expression to match the path to the srpm including the srpm name, capture the name as group if possible
* `chroots` list of change roots, default: `["fedora-25-x86_64"]`
* `enable_net` : enable only if you need the web for building the rpm from your srpm, default: `false`
* `max_n_bytes` : the maximum total number of bytes to push, default: `1000000000`

