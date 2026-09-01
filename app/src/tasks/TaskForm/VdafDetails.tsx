import { Props } from "./index.js";
import { SumBits } from "./SumBits.js";
import { HistogramBucketSelection } from "./HistogramBucketSelection.js";

export default function VdafDetails(props: Props) {
  switch (props.values.vdaf?.type) {
    case "sum":
      return <SumBits {...props} />;

    case "histogram":
      return <HistogramBucketSelection {...props} />;

    default:
      return <></>;
  }
}
