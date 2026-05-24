use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::usage::UsageDetails;
use crate::{
    api,
    i18n::use_i18n,
    types::{CookieLastError, CookieStatus, Reason, UselessCookie},
    utils::{self, format_iso, format_timestamp},
};

fn confirm_and_delete(cookie: String, deleting: RwSignal<bool>) {
    let i = use_i18n();
    let window = web_sys::window().unwrap();
    if !window
        .confirm_with_message(&i.t("cookieStatus.deleteConfirm"))
        .unwrap_or(false)
    {
        return;
    }
    deleting.set(true);
    let refresh = expect_context::<RwSignal<u32>>();
    spawn_local(async move {
        let _ = api::delete_cookie(&cookie).await;
        deleting.set(false);
        refresh.update(|v| *v += 1);
    });
}

#[component]
fn DeleteBtn(cookie: String) -> impl IntoView {
    let deleting = RwSignal::new(false);
    let c = cookie.clone();
    view! {
        <button
            class="icon-del"
            disabled=move || deleting.get()
            on:click=move |_| confirm_and_delete(c.clone(), deleting)
        >
            {move || if deleting.get() { "..." } else { "✕" }}
        </button>
    }
}

/// Short uppercase chip describing the raw reason variant.
fn reason_tag(reason: &Option<Reason>) -> String {
    let i = use_i18n();
    let key = match reason {
        None => "cookieStatus.status.tags.unknown",
        Some(Reason::Free) => "cookieStatus.status.tags.free",
        Some(Reason::Disabled) => "cookieStatus.status.tags.disabled",
        Some(Reason::Banned) => "cookieStatus.status.tags.banned",
        Some(Reason::Null) => "cookieStatus.status.tags.invalid",
        Some(Reason::NormalPro) => "cookieStatus.status.tags.normalPro",
        Some(Reason::Restricted(_)) => "cookieStatus.status.tags.restricted",
        Some(Reason::TooManyRequest(_)) => "cookieStatus.status.tags.rateLimit",
    };
    i.t(key)
}

const TAG_STYLE: &str =
    "display:inline-block;padding:0 0.4rem;border-radius:0.25rem;border:1px solid currentColor;\
     font-size:0.65rem;font-weight:600;letter-spacing:0.05em;";

/// Renders the account-warning chips (first/second/restricted) detected at
/// bootstrap. Returns `None` when no warnings are active so the section can
/// be omitted entirely.
fn warning_chips(cookie: &CookieStatus) -> Option<impl IntoView + use<>> {
    let i = use_i18n();
    let entries: Vec<(String, i64, &'static str)> = [
        (
            cookie.first_warning_at,
            "cookieStatus.status.warnings.firstWarning",
            "#fbbf24",
        ),
        (
            cookie.second_warning_at,
            "cookieStatus.status.warnings.secondWarning",
            "#f97316",
        ),
        (
            cookie.restricted_at,
            "cookieStatus.status.warnings.restricted",
            "#ef4444",
        ),
    ]
    .into_iter()
    .filter_map(|(opt, key, color)| opt.map(|ts| (i.t(key), ts, color)))
    .collect();

    if entries.is_empty() {
        return None;
    }

    let title = i.t("cookieStatus.status.warnings.title");
    Some(view! {
        <div>
            <div class="text-xs text-dim">{title}</div>
            <div class="row-sm" style="flex-wrap:wrap;gap:0.25rem;margin-top:0.25rem">
                {entries.into_iter().map(|(label, ts, color)| {
                    let style = format!("{TAG_STYLE}color:{color}");
                    view! {
                        <span style=style>
                            {format!("{label} {}", format_timestamp(ts))}
                        </span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    })
}

/// Renders the cookie's last non-rate-limit upstream HTTP error, or `None`
/// when no such error has been recorded.
fn last_error_block(err: &Option<CookieLastError>) -> Option<impl IntoView + use<>> {
    let i = use_i18n();
    let err = err.as_ref()?;
    let code = err.code.to_string();
    let header = i.tf("cookieStatus.status.lastError.code", &[("code", &code)]);
    let title = i.t("cookieStatus.status.lastError.title");
    let at_label = i.t("cookieStatus.status.lastError.at");
    let at = format_timestamp(err.at);
    let message = err.message.clone();
    Some(view! {
        <div>
            <div class="text-xs text-dim">{title}</div>
            <div class="text-xs" style="margin-top:0.25rem;color:#f87171">
                <span style=format!("{TAG_STYLE}color:#f87171;margin-right:0.4rem")>
                    {header}
                </span>
                <span class="text-mono" style="white-space:pre-wrap;word-break:break-word">
                    {message}
                </span>
                <div class="text-dim" style="margin-top:0.15rem">
                    {at_label}" "{at}
                </div>
            </div>
        </div>
    })
}

#[component]
pub fn ValidRow(cookie: CookieStatus) -> impl IntoView {
    let i18n = use_i18n();
    let cookie_str = StoredValue::new(cookie.cookie.clone());
    let masked = utils::mask_str(&cookie.cookie, 6);
    let expanded = RwSignal::new(false);

    let count_tokens_chip = cookie.count_tokens_allowed.map(|allowed| {
        let (label_key, color) = if allowed {
            ("cookieStatus.status.countTokensAllowed", "#4ade80")
        } else {
            ("cookieStatus.status.countTokensBlocked", "#f87171")
        };
        let style = format!("{TAG_STYLE}color:{color};margin-left:0.25rem");
        view! { <span style=style>{i18n.t(label_key)}</span> }
    });

    let warnings = warning_chips(&cookie);
    let last_err = last_error_block(&cookie.last_error);
    let details_cookie = cookie.clone();

    view! {
        <div class="cookie-row">
            <div class="flex-1">
                <div class="row-sm">
                    <span
                        class="text-mono text-xs"
                        style="color:#4ade80; cursor:pointer"
                        on:click=move |_| expanded.update(|e| *e = !*e)
                    >
                        {move || if expanded.get() { cookie_str.get_value() } else { masked.clone() }}
                    </span>
                    <button
                        class="icon-copy"
                        on:click=move |_| utils::copy_to_clipboard(cookie_str.get_value())
                    >"📋"</button>
                    {count_tokens_chip}
                </div>

                <details style="margin-top:0.25rem">
                    <summary>{i18n.t("cookieStatus.meta.summary")}</summary>
                    <div class="stack-sm" style="margin-top:0.5rem">
                        {warnings}
                        {last_err}
                        <UsageDetails cookie=details_cookie />
                    </div>
                </details>
            </div>
            <div class="row-sm">
                <span class="text-xs text-dim">{move || use_i18n().t("cookieStatus.status.available")}</span>
                <DeleteBtn cookie=cookie.cookie />
            </div>
        </div>
    }
}

#[component]
pub fn ExhaustedRow(cookie: CookieStatus) -> impl IntoView {
    let i18n = use_i18n();
    let masked = utils::mask_str(&cookie.cookie, 6);

    let reason_full = get_reason_text(&cookie.reason);
    let tag_text = reason_tag(&cookie.reason);
    let tag_style = format!("{TAG_STYLE}color:#facc15;margin-right:0.4rem");

    let cooldown = if let Some(ts) = cookie.reset_time {
        format!(
            "{}: {}",
            i18n.t("cookieStatus.status.cooldownFull"),
            format_timestamp(ts)
        )
    } else if let Some(ref s) = cookie.seven_day_sonnet_resets_at {
        format!(
            "{}: {}",
            i18n.t("cookieStatus.status.cooldownSonnet"),
            format_iso(s)
        )
    } else if let Some(ref s) = cookie.seven_day_resets_at {
        format!(
            "{}: {}",
            i18n.t("cookieStatus.status.cooldownFull"),
            format_iso(s)
        )
    } else {
        i18n.t("cookieStatus.status.unknownReset")
    };

    let warnings = warning_chips(&cookie);
    let last_err = last_error_block(&cookie.last_error);
    let details_cookie = cookie.clone();

    view! {
        <div class="cookie-row">
            <div class="flex-1">
                <div class="row-sm">
                    <span style=tag_style>{tag_text}</span>
                    <span class="text-mono text-xs truncate" style="color:#facc15">{masked}</span>
                </div>
                <div class="text-xs text-dim">{reason_full}</div>
                <details style="margin-top:0.25rem">
                    <summary>{i18n.t("cookieStatus.meta.summary")}</summary>
                    <div class="stack-sm" style="margin-top:0.5rem">
                        {warnings}
                        {last_err}
                        <UsageDetails cookie=details_cookie />
                    </div>
                </details>
            </div>
            <div class="row-sm">
                <span class="text-xs text-dim">{cooldown}</span>
                <DeleteBtn cookie=cookie.cookie />
            </div>
        </div>
    }
}

#[component]
pub fn InvalidRow(cookie: UselessCookie) -> impl IntoView {
    let masked = utils::mask_str(&cookie.cookie, 6);
    let reason = get_reason_text(&cookie.reason);
    let tag_text = reason_tag(&cookie.reason);
    let tag_style = format!("{TAG_STYLE}color:#f87171;margin-right:0.4rem");

    view! {
        <div class="cookie-row">
            <div class="flex-1 row-sm">
                <span style=tag_style>{tag_text}</span>
                <span class="text-mono text-xs truncate" style="color:#f87171">{masked}</span>
            </div>
            <div class="row-sm">
                <span class="text-xs text-dim">{reason}</span>
                <DeleteBtn cookie=cookie.cookie />
            </div>
        </div>
    }
}

fn get_reason_text(reason: &Option<Reason>) -> String {
    let i = use_i18n();
    let Some(r) = reason else {
        return i.t("cookieStatus.status.reasons.unknown");
    };
    match r {
        Reason::Free => i.t("cookieStatus.status.reasons.freAccount"),
        Reason::Disabled => i.t("cookieStatus.status.reasons.disabled"),
        Reason::Banned => i.t("cookieStatus.status.reasons.banned"),
        Reason::Null => i.t("cookieStatus.status.reasons.invalid"),
        Reason::NormalPro => "Normal Pro".into(),
        Reason::Restricted(ts) => {
            format!(
                "{} {}",
                i.t("cookieStatus.status.reasons.restricted"),
                format_timestamp(*ts)
            )
        }
        Reason::TooManyRequest(ts) => {
            format!(
                "{} {}",
                i.t("cookieStatus.status.reasons.rateLimited"),
                format_timestamp(*ts)
            )
        }
    }
}
