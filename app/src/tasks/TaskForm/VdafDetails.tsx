import { Props } from ".";
import { SumBits } from "./SumBits";
import { HistogramBucketSelection } from "./HistogramBucketSelection";

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
