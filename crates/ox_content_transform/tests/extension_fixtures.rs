//! Ensure performance workloads exercise their named extensions.
#[path = "../examples/extensions/cases.rs"]
mod extensions;
use ox_content_transform::transformer::MarkdownTransformer;

#[test]
fn native_extension_matrix_exercises_valid_output_and_absent_controls() {
    for case in extensions::cases() {
        let transformer = MarkdownTransformer::from_options(&case.options);
        let result = transformer.transform(&case.source);
        assert!(result.errors.is_empty(), "{}: {:?}", case.name, result.errors);
        assert!(result.html.contains(case.marker), "{} missing {}", case.name, case.marker);
        if case.name == "frontmatter" {
            assert!(result.frontmatter.contains("Extension benchmark"));
        }
        let absent = transformer.transform(extensions::PROSE);
        assert!(absent.errors.is_empty(), "{}: {:?}", case.name, absent.errors);
        assert!(absent.html.contains("Ordinary documentation"), "{}", case.name);
    }
}
