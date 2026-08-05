import type { JSX } from "solid-js";

export default function Alert(props: {
  type?: "info" | "success" | "warning";
  title?: string;
  children?: JSX.Element;
}) {
  return (
    <div class={`alert alert-${props.type ?? "info"}`}>
      {props.title ? <strong class="alert-title">{props.title}</strong> : null}
      <div class="alert-body">{props.children}</div>
    </div>
  );
}
