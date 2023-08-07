# Public task fetch api

Documentation state: Proposal

## Motivation and Context

Currently, we instruct people to copy-and-paste code like this in order to use divviup-ts:
```js
import DAPClient from "@divviup/dap";
const client = new DAPClient({
  "type": "sum",
  "bits": 8,
  "taskId": "5YXXYPFzt1a8cuo8AlKqs6oKbt3FIrkn3Q8JseJKRYs",
  "leader": "https://dap.xxqbi.example/",
  "helper": "https://dap.xxqbi.example/",
  "timePrecisionSeconds": 1080
});
```

Accidentally modifying any of these plaintext values would make report generation fail.

## Proposed improvement

We add an infinitely cacheable (`Cache-Control: public, max-age=604800`) endpoint `GET {divviup-api url}/api/tasks/:task_id`. When a task is found with the provided task identifier, the divviup-api server responds with the following json:

```json
{
  "id": "5YXXYPFzt1a8cuo8AlKqs6oKbt3FIrkn3Q8JseJKRYs",
  "vdaf": {
    "type": "sum",
    "bits": 8
  },
  "leader": "https://dap.xxqbi.example/",
  "helper": "https://dap.xxqbi.example/",
  "time_precision_seconds": 1080,
  "protocol": "DAP-04"
}
```

and the client can be configured like:

```js
import DivviupClient from "@divviup/client";
const client = new DivviupClient("https://api.divviup.org/api/tasks/5YXXYPFzt1a8cuo8AlKqs6oKbt3FIrkn3Q8JseJKRYs");
```

or, optionally, the following shortcut is also supported:

```js
import DivviupClient from "@divviup/client";
const client = new DivviupClient("5YXXYPFzt1a8cuo8AlKqs6oKbt3FIrkn3Q8JseJKRYs");
```

## Redirection

In the future, if a task is replaced by a new task, the client would follow a http redirect at this endpoint. This might happen if, for example, the DAP version was sunsetted, or as a migration path at the end of task expiration. As a result, we might want to have slightly less permanent cache-control settings.
