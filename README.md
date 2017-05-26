# copr resource for concourse

https://copr.fedoraproject.org/

http://copr-rest-api.readthedocs.io/en/latest/index.html

## source

* `username` copr username you use to log in on the web interface
* `login` copr login
* `token` copr token used to login in combination with `login`
* `url` copr url, usually https://copr.fedorainfracloud.org or whatever you see while logged in at https://copr.fedoraproject.org/api/
* `project_id` the associated project_id `curl -u "$LOGIN:$TOKEN" -X GET https://copr.fedorainfracloud.org/api_2/projects?name=$PROJECT_NAME | jq -r ".projects[].project.id"` to
* `regex` regular expression to match the path to the srpm including the srpm name, capture the name as group if possible

### check

`unimpletented!()`, always returns `[]` which means no new version

### in

`unimpletented!()`, always crashes

### out

TODO
