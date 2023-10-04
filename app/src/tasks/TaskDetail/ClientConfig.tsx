import { useLoaderData } from "react-router-dom";
import Col from "react-bootstrap/Col";
import { Tab, Tabs } from "react-bootstrap";
import { Task, Aggregator } from "../../ApiClient";
import Card from "react-bootstrap/Card";
import "@github/relative-time-element";
import { Copy, OutLink, usePromiseAll } from "../../util";
import SyntaxHighlighter from "react-syntax-highlighter";
import { github as syntaxStyle } from "react-syntax-highlighter/dist/esm/styles/hljs";

export default function ClientConfig() {
  return (
    <Col md="6">
      <Card className="my-3">
        <Card.Body>
          <Card.Title>Client Config</Card.Title>
          <Tabs defaultActiveKey="ts" id="client-config" className="mb-3">
            <Tab eventKey="ts" title="TypeScript/JavaScript">
              <TsClientConfig />
            </Tab>
            <Tab eventKey="rs" title="Rust">
              <RsClientConfig />
            </Tab>
          </Tabs>
        </Card.Body>
      </Card>
    </Col>
  );
}

function RsClientConfig() {
  return (
    <OutLink href="https://lib.rs/crates/janus_client">
      View Documentation
    </OutLink>
  );
}

function TsClientConfig() {
  const { task, leaderAggregator, helperAggregator } = useLoaderData() as {
    task: Promise<Task>;
    leaderAggregator: Promise<Aggregator>;
    helperAggregator: Promise<Aggregator>;
  };

  const taskConfig = usePromiseAll(
    [task, leaderAggregator, helperAggregator],
    ([task, leader, helper]) => ({
      ...task.vdaf,
      taskId: task.id,
      leader: leader.dap_url,
      helper: helper.dap_url,
      timePrecisionSeconds: task.time_precision_seconds,
    }),
    {
      type: null,
      taskId: null,
      leader: null,
      helper: null,
      timePrecisionSeconds: null,
    } as object,
  );

  const taskArgs = JSON.stringify(taskConfig, null, 2).replace(
    /"([^"]+)":/g,
    "$1:",
  );

  const contents = `import Task from "@divviup/dap";

const task = new Task(${taskArgs});

await task.sendMeasurement(...); // your measurement here`;

  return (
    <>
      <p>
        Copy and paste this code to use{" "}
        <OutLink href="https://github.com/divviup/divviup-ts">
          divviup-ts
        </OutLink>
      </p>

      <Copy clipboardContents={contents}>
        {(copy) => (
          <div
            onClick={copy}
            style={copy && { cursor: "pointer" }}
            className="my-3"
          >
            <SyntaxHighlighter
              language="javascript"
              style={syntaxStyle}
              showLineNumbers
            >
              {contents}
            </SyntaxHighlighter>
          </div>
        )}
      </Copy>

      <p>
        <OutLink href="https://divviup.github.io/divviup-ts/">
          View Documentation
        </OutLink>
      </p>
    </>
  );
}
