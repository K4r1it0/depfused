//! AST-based JavaScript parser using oxc_parser.

use crate::parser::normalize_package_name;
use crate::types::{Confidence, ExtractionMethod, Package, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashSet;
use tracing::{debug, trace};

/// AST-based parser for extracting package names from JavaScript.
#[derive(Clone)]
pub struct AstParser {
    /// Whether to include low-confidence extractions.
    include_low_confidence: bool,
}

impl AstParser {
    /// Create a new AST parser.
    pub fn new(include_low_confidence: bool) -> Self {
        Self {
            include_low_confidence,
        }
    }

    /// Parse JavaScript content and extract package references.
    pub fn parse(&self, content: &str, source_url: &str) -> Result<Vec<Package>> {
        let allocator = Allocator::default();
        let source_type = SourceType::default()
            .with_module(true)
            .with_jsx(true);

        let parser_result = Parser::new(&allocator, content, source_type).parse();

        // We continue even with parse errors (common in minified code)
        if !parser_result.errors.is_empty() {
            trace!(
                "Parse had {} errors for {}, continuing...",
                parser_result.errors.len(),
                source_url
            );
        }

        let program = parser_result.program;

        // Extract using visitor
        let mut visitor = PackageVisitor::new(source_url.to_string(), self.include_low_confidence);
        visitor.visit_program(&program);

        // Also extract from comments
        let comment_packages = self.extract_from_comments(content, source_url);
        visitor.packages.extend(comment_packages);

        // Also look for string patterns that might be packages
        let string_packages = self.extract_from_strings(content, source_url);
        visitor.packages.extend(string_packages);

        let packages: Vec<Package> = visitor.packages.into_iter().collect();
        debug!(
            "Extracted {} packages from AST: {}",
            packages.len(),
            source_url
        );

        Ok(packages)
    }

    /// Extract package names from comments.
    fn extract_from_comments(&self, content: &str, source_url: &str) -> HashSet<Package> {
        let mut packages = HashSet::new();

        // First, extract only comment text from the content
        // This prevents matching @scope/name patterns in actual import statements
        let mut comment_text = String::new();

        // Pattern for block comments: /* ... */
        if let Ok(block_comment_re) = regex::Regex::new(r"/\*[\s\S]*?\*/") {
            for cap in block_comment_re.find_iter(content) {
                comment_text.push_str(cap.as_str());
                comment_text.push('\n');
            }
        }

        // Pattern for line comments: // ...
        if let Ok(line_comment_re) = regex::Regex::new(r"//[^\n]*") {
            for cap in line_comment_re.find_iter(content) {
                comment_text.push_str(cap.as_str());
                comment_text.push('\n');
            }
        }

        // Only extract @scope/package patterns from within comment text
        // Pattern: /*! @scope/package v1.2.3 */
        // Pattern: // Built with @company/tool
        let package_pattern = regex::Regex::new(r"@([\w-]+)/([\w.-]+)").unwrap();

        for cap in package_pattern.captures_iter(&comment_text) {
            if let (Some(scope), Some(name)) = (cap.get(1), cap.get(2)) {
                let full_name = format!("@{}/{}", scope.as_str(), name.as_str());
                if let Some(normalized) = normalize_package_name(&full_name) {
                    packages.insert(Package {
                        name: normalized,
                        extraction_method: ExtractionMethod::Comment,
                        source_url: source_url.to_string(),
                        confidence: Confidence::Medium,
                    });
                }
            }
        }

        packages
    }

    /// Extract package names from error message strings.
    fn extract_from_strings(&self, content: &str, source_url: &str) -> HashSet<Package> {
        let mut packages = HashSet::new();

        if !self.include_low_confidence {
            return packages;
        }

        // Pattern: "Cannot find module '@scope/pkg'"
        // Pattern: "Error in @company/utils"
        let error_patterns = [
            r#"Cannot find module ['"](@[\w-]+/[\w.-]+|[\w.-]+)['"]"#,
            r#"Error in ['"]?(@[\w-]+/[\w.-]+)['"]?"#,
            r#"Module not found.*['"](@[\w-]+/[\w.-]+)['"]"#,
        ];

        for pattern in error_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(content) {
                    if let Some(pkg) = cap.get(1) {
                        if let Some(normalized) = normalize_package_name(pkg.as_str()) {
                            packages.insert(Package {
                                name: normalized,
                                extraction_method: ExtractionMethod::ErrorMessage,
                                source_url: source_url.to_string(),
                                confidence: Confidence::Low,
                            });
                        }
                    }
                }
            }
        }

        packages
    }
}

/// Visitor for extracting package names from AST.
struct PackageVisitor {
    packages: HashSet<Package>,
    source_url: String,
    include_low_confidence: bool,
}

impl PackageVisitor {
    fn new(source_url: String, include_low_confidence: bool) -> Self {
        Self {
            packages: HashSet::new(),
            source_url,
            include_low_confidence,
        }
    }

    fn add_package(&mut self, name: &str, method: ExtractionMethod, confidence: Confidence) {
        if confidence == Confidence::Low && !self.include_low_confidence {
            return;
        }

        if let Some(normalized) = normalize_package_name(name) {
            self.packages.insert(Package {
                name: normalized,
                extraction_method: method,
                source_url: self.source_url.clone(),
                confidence,
            });
        }
    }

    fn extract_from_string_literal(&mut self, value: &str, method: ExtractionMethod) {
        self.add_package(value, method, Confidence::High);
    }
}

impl<'a> Visit<'a> for PackageVisitor {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let source = decl.source.value.as_str();
        self.extract_from_string_literal(source, ExtractionMethod::Import);
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let source = decl.source.value.as_str();
        self.extract_from_string_literal(source, ExtractionMethod::Import);
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(ref source) = decl.source {
            self.extract_from_string_literal(source.value.as_str(), ExtractionMethod::Import);
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // Check for require('package')
        if let Expression::Identifier(id) = &expr.callee {
            if id.name == "require" {
                if let Some(Argument::StringLiteral(lit)) = expr.arguments.first() {
                    self.extract_from_string_literal(lit.value.as_str(), ExtractionMethod::Require);
                }
            }
        }

        walk::walk_call_expression(self, expr);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        // Dynamic import('package')
        if let Expression::StringLiteral(lit) = &expr.source {
            self.extract_from_string_literal(lit.value.as_str(), ExtractionMethod::DynamicImport);
        }
        walk::walk_import_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_imports() {
        let parser = AstParser::new(false);
        let js = r#"
            import lodash from 'lodash';
            import { useState } from 'react';
            import * as utils from '@company/utils';
        "#;

        let packages = parser.parse(js, "test.js").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"react"));
        assert!(names.contains(&"@company/utils"));
    }

    #[test]
    fn test_parse_require() {
        let parser = AstParser::new(false);
        let js = r#"
            const fs = require('fs');
            const lodash = require('lodash');
            const internal = require('@internal/auth');
        "#;

        let packages = parser.parse(js, "test.js").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"@internal/auth"));
        // fs is a built-in, should be excluded
        assert!(!names.contains(&"fs"));
    }

    #[test]
    fn test_parse_dynamic_import() {
        let parser = AstParser::new(false);
        let js = r#"
            const loadModule = async () => {
                const mod = await import('lodash');
                return mod;
            };
        "#;

        let packages = parser.parse(js, "test.js").unwrap();
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();

        assert!(names.contains(&"lodash"));
    }

    #[test]
    fn test_skip_relative_imports() {
        let parser = AstParser::new(false);
        let js = r#"
            import local from './local';
            import parent from '../parent';
            import absolute from '/absolute/path';
        "#;

        let packages = parser.parse(js, "test.js").unwrap();
        assert!(packages.is_empty());
    }
}
