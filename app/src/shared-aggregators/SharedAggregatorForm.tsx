import { FormikHelpers } from "formik";
import ApiClient, { NewAggregator, formikErrors } from "../ApiClient";
import { AggregatorForm } from "../aggregators/AggregatorForm";
import { useNavigate, useRevalidator } from "react-router-dom";
import { ApiClientContext } from "../ApiClientContext";
import React from "react";

async function submit(
  apiClient: ApiClient,
  newAggregator: NewAggregator,
  actions: FormikHelpers<NewAggregator>,
  revalidate: () => void,
) {
  try {
    let aggregator = await apiClient.createSharedAggregator(newAggregator);

    if ("error" in aggregator) {
      actions.setErrors(formikErrors(aggregator.error));
    } else {
      actions.resetForm();
      revalidate();
    }
  } catch (e) {
    console.log(e);
  }
}

export default function SharedAggreatorForm() {
  const navigate = useNavigate();
  const apiClient = React.useContext(ApiClientContext);
  const { revalidate } = useRevalidator();
  const handleSubmit = React.useCallback(
    (values: NewAggregator, actions: FormikHelpers<NewAggregator>) =>
      submit(apiClient, values, actions, revalidate),
    [apiClient, navigate, revalidate],
  );

  return (
    <>
      <h1>New Shared Aggregator</h1>
      <AggregatorForm handleSubmit={handleSubmit} showIsFirstParty />
    </>
  );
}
