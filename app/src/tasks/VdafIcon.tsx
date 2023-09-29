import { Task } from "../ApiClient";
import {
  FileEarmarkBarGraph,
  FileEarmarkBarGraphFill,
  FileEarmarkBinary,
  FileEarmarkBinaryFill,
  FileEarmarkPlus,
  FileEarmarkPlusFill,
} from "react-bootstrap-icons";

export function VdafIcon({
  task,
  fill = false,
}: {
  task: Task;
  fill?: boolean;
}) {
  switch (task.vdaf.type.toLowerCase()) {
    case "sum":
      return fill ? <FileEarmarkPlusFill /> : <FileEarmarkPlus />;
    case "histogram":
      return fill ? <FileEarmarkBarGraphFill /> : <FileEarmarkBarGraph />;
    case "count":
      return fill ? <FileEarmarkBinaryFill /> : <FileEarmarkBinary />;
    default:
      return <></>;
  }
}
