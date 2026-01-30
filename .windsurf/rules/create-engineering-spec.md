---
trigger: manual
description: Guide for creating detailed engineering tickets with comprehensive requirements, acceptance criteria, and implementation task lists
---

# Rule: Generating Engineering Tickets with Task Lists

## Goal

To guide an AI assistant in creating detailed Engineering Tickets in Markdown format, based on an initial user prompt. The ticket should include comprehensive requirements, acceptance criteria, and a detailed task breakdown for implementation. This will be a liviing document we can update while we work on implementing the spec.

## Process

1. **Receive Initial Prompt:** The user provides a brief description or request for a new feature, bug fix, or technical task (e.g., "@create-engineering-spec implement user authentication system").

2. **Determine Ticket Identifier & File Number:**
   - **Ask the user:** Prompt for the Jira ticket number associated with this spec (e.g., `CFONE-1234`).  
     Example AI prompt:
     > "What is the Jira ticket number for this spec? For example: CFONE-1234. If there’s no associated Jira ticket, I’ll use a NOJIRA code instead."
   - **If provided:** Use the Jira ticket number exactly as given.
   - **If not provided:**
     - Check the `/specs` directory for existing files starting with the pattern `NOJIRA-[number]-spec-*.md`.
     - Identify the highest existing `NOJIRA` number.
     - Increment by 1 to generate the new identifier (e.g., `NOJIRA-01`, `NOJIRA-02`, etc.). If no `NOJIRA` files exist, start with `NOJIRA-01`.

3. **Ask Clarifying Questions:** Before writing the ticket, the AI _must_ ask clarifying questions to gather sufficient technical and functional detail. The goal is to understand both the "what" and "why" of the task, plus any technical constraints.

- Repeat the process with questions until you get approval or no more questions remain unanswered.

4. **Generate Engineering Ticket:** Based on the initial prompt and the user's answers to the clarifying questions, generate a ticket using the structure outlined below.

5. **Phase 1: Generate Parent Tasks:** Create the main, high-level tasks required to implement the feature. **The tasks must be ordered to follow a logical development flow that enables incremental progress and fast feedback.** Present these tasks to the user and inform them: "I have generated the high-level tasks based on the requirements. Ready to generate the detailed sub-tasks? Respond with 'yes' or 'y' to proceed."

6. **Wait for Confirmation:** Pause and wait for the user to respond with "yes" or "y".

7. **Phase 2: Generate Sub-Tasks:** Once the user confirms, break down each parent task into smaller, actionable sub-tasks necessary to complete the parent task.

8. **Save Ticket:**  
   Save the generated document as `[ticket-id]-spec-[feature-name].md` inside the `/specs` directory, where:
   - `[ticket-id]` is either the Jira ticket number (e.g., `CFONE-1234`) or the `NOJIRA-XX` identifier determined in step 2.
   - `[feature-name]` is derived from the user's prompt (e.g., `CFONE-1234-spec-user-authentication.md` or `NOJIRA-01-spec-user-authentication.md`).

## Clarifying Questions (Examples)

Example format:

1. Top-level question 1?
2. Top-level question 2?
   2.1. Sub-question related to question 2?
   2.2. Another sub-question related to question 2?
3. Top-level question 3?

The AI should adapt its questions based on the prompt, but here are some common areas to explore:

- **Problem/Task:** "What specific problem does this solve or what functionality needs to be implemented?"
- **Technical Context:** "Are there any existing systems, APIs, or components this needs to integrate with?"
- **Scope & Boundaries:** "What should be included in this ticket vs. what should be separate tickets?"
- **User Impact:** "Who will be affected by this change and how?"
- **Acceptance Criteria:** "How will we know when this is complete and working correctly?"
- **Technical Constraints:** "Are there any technical limitations, performance requirements, or security considerations?"
- **Dependencies:** "Does this depend on other work being completed first?"
- **Testing Requirements:** "What specific testing scenarios should be considered?"
- **Implementation Approach:** "Are there any preferred technical approaches or patterns to follow?"

## Engineering Ticket Structure

The generated ticket should include the following sections:

1. **Title:** Clear, descriptive title starting with "Engineering Spec:"
2. **Description:** Detailed explanation of the feature or task to be implemented
3. **Technical Context:** Relevant technical background, architecture considerations, or system constraints
4. **Implementation Details:** Proposed implementation approach or technical considerations
5. **Acceptance Criteria:** Either as a numbered list OR in Given-When-Then format (ask user for preference)
6. **Testing Considerations:** Specific testing requirements and scenarios
7. **Dependencies:** External dependencies, blocked work, or prerequisite tasks
8. **Resources:** Links to design documents, API docs, related tickets, or other references
9. **Relevant Files:** List of files that will need to be created or modified
10. **Implementation Tasks:** Hierarchical task breakdown with parent tasks and sub-tasks

## Acceptance Criteria

```
## Acceptance Criteria
1. [Criterion 1]
2. [Criterion 2]
```

## Task List Structure

The task breakdown should follow a **strategic development flow** based on the type of feature:

### For API/Backend Features:

1. **Endpoint Structure & Routing** - Get the basic API accessible first
2. **Request/Response Validation** - Ensure proper data flow
3. **Core Business Logic** - Implement the complex functionality
4. **Testing** - Comprehensive validation
5. **Documentation & Finalization** - Complete the feature

### For Frontend Features:

1. **Component Structure** - Create basic component files
2. **UI Scaffolding** - Basic layout and structure
3. **State Management** - Data flow and state handling
4. **UI Implementation** - Complete visual implementation
5. **Integration** - Connect to backend/data sources
6. **Testing** - Comprehensive validation

### Benefits:

- Enables early testing of basic connectivity
- Allows incremental development with fast feedback
- Makes debugging easier by isolating layers
- Follows "make it work, then make it right" principle

### Task Format:

```
## Relevant Files

- `path/to/potential/file1.ts` - Brief description of why this file is relevant
- `path/to/file1.test.ts` - Unit tests for `file1.ts`
- `path/to/another/file.tsx` - Brief description
- `path/to/another/file.test.tsx` - Unit tests for `another/file.tsx`

### Notes

- Unit tests should be placed in the test directory PROJECT_ROOT/test
- The test directoy copies the src directoy project structure

Example
```

PROJECT_ROOT/
├── src/
│ └── utils/
│ ├── utils.ts
│ └── ...
├── test/
│ └── utils/
│ ├── utils.test.ts

```


## Implementation Tasks

- [ ] 1.0 Parent Task Title
  - [ ] 1.1 [Sub-task description 1.1]
  - [ ] 1.2 [Sub-task description 1.2]
- [ ] 2.0 Parent Task Title
  - [ ] 2.1 [Sub-task description 2.1]
- [ ] 3.0 Parent Task Title
```

## Target Audience

Assume the primary reader of the task list is a **junior developer** who will implement the feature and needs extra details

## Output

- **Format:** Markdown (`.md`)
- **Location:** `/specs/`
- **Filename:** `[number]-spec-[feature-name].md` (e.g., `06-spec-user-authentication.md`)

```

## Interaction Model

The process explicitly requires a pause after generating parent tasks to get user confirmation ("yes" or "y") before proceeding to generate the detailed sub-tasks. This ensures the high-level plan aligns with user expectations before diving into details.

## Open Questions

1. **Question Category**: Specific question that needs clarification
2. **Another Category**: Another question requiring resolution during development

## Examples

For examples of Specs look at the specs/* directoy for existing specs.

## Final Instructions

1. **Do NOT** start implementing the ticket or any code
2. **Do NOT** create the actual ticket file until after asking clarifying questions
3. **Take the user's answers** to the clarifying questions and create a comprehensive ticket
4. **Focus on technical precision** while maintaining clarity for the development team
5. **Consider the full development lifecycle** including testing, deployment
6. **Generate parent tasks first** and wait for user confirmation before creating sub-tasks
7. **Order tasks logically** to enable incremental progress and fast feedback
8. **Document open questions** to prevent blocking during implementation
```
