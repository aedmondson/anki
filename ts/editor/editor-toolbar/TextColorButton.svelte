<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { bridgeCommand } from "@tslib/bridgecommand";
    import * as tr from "@tslib/ftl";
    import { getPlatformString } from "@tslib/shortcuts";
    import { removeStyleProperties } from "@tslib/styling";
    import { singleCallback } from "@tslib/typing";
    import { setFormat } from "editor/old-editor-adapter";
    import { onMount } from "svelte";

    import IconButton from "../../components/IconButton.svelte";
    import Shortcut from "../../components/Shortcut.svelte";
    import type { FormattingNode, MatchType } from "../../domlib/surround";
    import { withFontColor } from "../helpers";
    import { chevronDown } from "../icons";
    import { surrounder } from "../rich-text-input";
    import ColorPicker from "./ColorPicker.svelte";
    import { context as editorToolbarContext } from "./EditorToolbar.svelte";
    import { textColorIcon } from "./icons";
    import WithColorHelper from "./WithColorHelper.svelte";

    export let color: string;

    $: transformedColor = transformColor(color);

    /**
     * The DOM will transform colors such as "#ff0000" to "rgb(255, 0, 0)".
     */
    function transformColor(color: string): string {
        const span = document.createElement("span");
        span.style.setProperty("color", color);
        return span.style.getPropertyValue("color");
    }

    function matcher(
        element: HTMLElement | SVGElement,
        match: MatchType<string>,
    ): void {
        if (
            withFontColor(element, (color: string): void => {
                if (color) {
                    match.setCache(color);
                    match.remove();
                }
            })
        ) {
            return;
        }

        const value = element.style.getPropertyValue("color");

        if (value.length === 0) {
            return;
        }

        match.setCache(value);
        match.clear((): void => {
            if (
                removeStyleProperties(element, "color") &&
                element.className.length === 0
            ) {
                match.remove();
            }
        });
    }

    function merger(
        before: FormattingNode<string>,
        after: FormattingNode<string>,
    ): boolean {
        return before.getCache(transformedColor) === after.getCache(transformedColor);
    }

    function formatter(node: FormattingNode<string>): boolean {
        const extension = node.extensions.find(
            (element: HTMLElement | SVGElement): boolean => element.tagName === "SPAN",
        );
        const color = node.getCache(transformedColor);

        if (extension) {
            extension.style.setProperty("color", color);
            return false;
        }

        const span = document.createElement("span");
        span.style.setProperty("color", color);
        node.range.toDOMRange().surroundContents(span);
        return true;
    }

    const key = "textColor";

    const format = {
        matcher,
        merger,
        formatter,
    };

    const namedFormat = {
        key,
        name: tr.editingTextColor(),
        show: true,
        active: true,
    };

    const { removeFormats } = editorToolbarContext.get();
    removeFormats.update((formats) => [...formats, namedFormat]);

    function setTextColor(): void {
        setFormat("foreColor", transformedColor);
    }

    const setCombination = "F7";
    const pickCombination = "F8";

    let disabled: boolean;

    onMount(() =>
        singleCallback(
            surrounder.active.subscribe((value) => (disabled = !value)),
            surrounder.registerFormat(key, format),
        ),
    );
</script>

<WithColorHelper {color} let:colorHelperIcon let:setColor>
    <IconButton
        tooltip="{tr.editingTextColor()} ({getPlatformString(setCombination)})"
        {disabled}
        on:click={setTextColor}
        --border-left-radius="5px"
    >
        {@html textColorIcon}
        {@html colorHelperIcon}
    </IconButton>
    <Shortcut keyCombination={setCombination} on:action={setTextColor} />

    <IconButton
        tooltip="{tr.editingChangeColor()} ({getPlatformString(pickCombination)})"
        {disabled}
        widthMultiplier={0.5}
        iconSize={120}
    >
        {@html chevronDown}
        <ColorPicker
            keyCombination={pickCombination}
            on:input={(event) => {
                color = setColor(event);
                bridgeCommand(`lastTextColor:${color}`);
            }}
            on:change={() => setTextColor()}
        />
    </IconButton>
</WithColorHelper>
