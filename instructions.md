---
pagetitle: Instructions
...

# Instructions

This API exposes three endpoints:

- `/api`: This documentation.
- `/api/tasks`
- `/api/tasks/<task_id>`

## `/api/tasks`

This endpoint responds to HTTP `GET` requests with a JSON with the following fields:

- `task_id`: corresponds to a `string` identifying the current task.
- `token`: corresponds to a `string` used to authenticate against the API when submitting results, see documentation for `/api/tasks/<task_id>` endpoint below.
- `op`: a `string` identifying the desired operation for this task, either `Max` or `Min`.
- `args`: an array (of arbitrary length) of integers (valued between -127 and 127).

## `/api/tasks/<task_id>`

This endpoint responds to HTTP `POST` requests representing a response to the given `task_id`.
Each request must be a valid JSON with a single field `result`, corresponding to an integer representing the attempted answer to the operation specified in the task response obtained from `/api/tasks`.
Additionally, each request to this endpoint must include an `Authorization: Bearer <token>` header carrying the authorization token provided in the response obtained from `/api/tasks`.

Responses from this endpoint will consist of a JSON object with the following fields:

- `success`: `boolean` indicating if the request was successful or not.
- `error`: optional `string` describing any error that occurred during processing, only included when `success` is `false`.
- `received`: optional `int` corresponding to the received `result` value from the request.
- `expected`: optional `int` corresponding to the expected value for the task.

## Examples

Getting a task and posting a valid, correct response:

```text
GET /api/tasks
```
```json
{
  "task_id":"546b1685-e77f-42a3-92f6-afc94032ebb2",
  "token":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MTAyNTc4MjIsImV4cCI6MTcxMDI1ODEyMiwibmJmIjoxNzEwMjU3ODIyfQ.rb0TJc895agcmp3KfMHlTQBiEtLTa_GonBcWL8-97P8",
  "op":"Max",
  "args":[-12,-1]
}
```
```text
POST /api/tasks/546b1685-e77f-42a3-92f6-afc94032ebb2
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MTAyNTc4MjIsImV4cCI6MTcxMDI1ODEyMiwibmJmIjoxNzEwMjU3ODIyfQ.rb0TJc895agcmp3KfMHlTQBiEtLTa_GonBcWL8-97P8
{"result": -1}
```
```json
{
  "success": true,
  "error": null,
  "received": -1,
  "expected": -1
}
```

---

Posting an incorrect response for the same task:

```text
POST /api/tasks/546b1685-e77f-42a3-92f6-afc94032ebb2
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MTAyNTc4MjIsImV4cCI6MTcxMDI1ODEyMiwibmJmIjoxNzEwMjU3ODIyfQ.rb0TJc895agcmp3KfMHlTQBiEtLTa_GonBcWL8-97P8
{"result": -12}
```
```json
{
  "success": false,
  "error": "incorrect result",
  "received": -12,
  "expected": -1
}
```

---

Posting a result without an authorization header:

```text
POST /api/tasks/546b1685-e77f-42a3-92f6-afc94032ebb2
{"result": -12}
```
```json
{
  "success": false,
  "error": "must provide an Authorization header with a valid token",
  "received": null,
  "expected": null
}
```

---

Posting an invalid request:

```text
POST /api/tasks/546b1685-e77f-42a3-92f6-afc94032ebb2
Content-Type: plain/text
"not a json"
```
```json
415 Unsupported Media Type
Expected request with `Content-Type: application/json`
```

Note that in this case (and some others), you can't rely on obtaining a valid JSON from the API, and should therefore ALWAYS first check that the status code is `200 OK` before attempting to parse the response body.