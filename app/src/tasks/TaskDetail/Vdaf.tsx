import { Task } from "../../ApiClient";
import ListGroup from "react-bootstrap/ListGroup";

export default function Vdaf({ task }: { task: Task }) {
  switch (task.vdaf.type) {
    case "sum":
      return (
        <ListGroup.Item>
          Sum maximum value: {Math.pow(2, task.vdaf.bits)}
        </ListGroup.Item>
      );
    case "histogram":
      return (
        <>
          <ListGroup.Item>
            Histogram Buckets: {task.vdaf.buckets.join(", ")}
          </ListGroup.Item>
        </>
      );

    case "count":
      return <ListGroup.Item>Count</ListGroup.Item>;
  }
}
