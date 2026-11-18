//! Call signatures (`(a: string): string`) and construct signatures
//! (`new (a: string): object`) of interfaces and type literals.
//!
//! These members have no name, so the renderers label them by kind instead,
//! as TypeDoc does.

use crate::model::ApiDocMember;
use crate::string_builder::StringBuilder;

/// Member kind of a call signature.
pub(super) const CALL_SIGNATURE: &str = "callSignature";
/// Member kind of a construct signature.
pub(super) const CONSTRUCT_SIGNATURE: &str = "constructSignature";

/// Whether the member is a call or construct signature.
pub(super) fn is_signature_member(member: &ApiDocMember) -> bool {
    matches!(member.kind.as_str(), CALL_SIGNATURE | CONSTRUCT_SIGNATURE)
}

/// Label shown where the member's name would be: TypeDoc's `Call Signature`
/// heading, and `Constructor` for construct signatures.
pub(super) fn signature_member_label(member: &ApiDocMember) -> Option<&'static str> {
    match member.kind.as_str() {
        CALL_SIGNATURE => Some("Call Signature"),
        CONSTRUCT_SIGNATURE => Some("Constructor"),
        _ => None,
    }
}

/// Name to show for a member: the label of a call or construct signature,
/// otherwise its own name.
pub(super) fn member_display_name(member: &ApiDocMember) -> &str {
    signature_member_label(member).unwrap_or(&member.name)
}

/// Anchor slug of the label (`call-signature`, `constructor`).
pub(super) fn signature_member_anchor(member: &ApiDocMember) -> Option<&'static str> {
    match member.kind.as_str() {
        CALL_SIGNATURE => Some("call-signature"),
        CONSTRUCT_SIGNATURE => Some("constructor"),
        _ => None,
    }
}

/// The signature to show, without a trailing `;`. With an owner, it is named
/// after the owner as TypeDoc does: `Name(a: string): string` and
/// `new Name(a: string): object`.
pub(super) fn signature_member_text(member: &ApiDocMember, owner: Option<&str>) -> Option<String> {
    if !is_signature_member(member) {
        return None;
    }
    let signature = member.signature.as_deref()?.trim().trim_end_matches(';').trim_end();
    if signature.is_empty() {
        return None;
    }
    let Some(owner) = owner.filter(|owner| !owner.is_empty()) else {
        return Some(signature.to_string());
    };

    let mut out = StringBuilder::with_capacity(owner.len() + signature.len() + 4);
    if member.kind == CONSTRUCT_SIGNATURE {
        out.push_str("new ");
        out.push_str(owner);
        out.push_str(signature.strip_prefix("new").map_or(signature, str::trim_start));
    } else {
        out.push_str(owner);
        out.push_str(signature);
    }
    Some(out.into_string())
}

/// The signature as a code block line: [`signature_member_text`] with a `;`.
pub(super) fn signature_member_code(member: &ApiDocMember, owner: Option<&str>) -> Option<String> {
    let mut code = signature_member_text(member, owner)?;
    code.push(';');
    Some(code)
}
