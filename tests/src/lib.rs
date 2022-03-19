#[cfg(test)]
mod tests {
    use weblab::weblab_assignment_info;

    mod m_simple_conversion {
        use weblab::weblab;

        #[weblab(programming_assignment)]
        /// description
        #[weblab(title = "yyy")]
        mod xxx {
            #[weblab(solution)]
            mod solution {}

            #[weblab(test)]
            mod test {}

            #[weblab(library)]
            mod library {}
        }
    }

    mod m_no_title {
        use weblab::weblab;

        #[weblab(programming_assignment)]
        mod aaa {
            #[weblab(solution)]
            mod solution {}

            #[weblab(test)]
            mod test {}

            #[weblab(library)]
            mod library {}
        }
    }

    #[test]
    fn simple_conversion() {
        let a = weblab_assignment_info!(m_simple_conversion);
        assert_eq!(a.title(), "yyy");
        assert_eq!(a.assignment_text(), "description");
    }

    #[test]
    fn no_title() {
        let a = weblab_assignment_info!(m_no_title);
        assert_eq!(a.title(), "aaa");
    }
}
