use serde_json::{Value, json};

use crate::AppState;

pub fn openapi_document(app: &AppState) -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "open Session Manager Local API",
            "version": app.version,
            "description": "Local read-only API for browsing OSM session inventory and details."
        },
        "servers": [
            {
                "url": "http://127.0.0.1:43210",
                "description": "Default local server"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": {
                        "200": {
                            "description": "Server health status",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/HealthResponse" },
                                    "example": {
                                        "status": "ok",
                                        "appName": app.app_name,
                                        "version": app.version
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/openapi.json": {
                "get": {
                    "summary": "OpenAPI document",
                    "responses": {
                        "200": {
                            "description": "Current OpenAPI schema"
                        }
                    }
                }
            },
            "/api/v1/sessions": {
                "get": {
                    "summary": "List session inventory",
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "$ref": "#/components/parameters/AssistantFilter" },
                        { "$ref": "#/components/parameters/Limit" },
                        { "$ref": "#/components/parameters/Offset" },
                        { "$ref": "#/components/parameters/SortBy" },
                        { "$ref": "#/components/parameters/Descending" }
                    ],
                    "responses": {
                        "200": {
                            "description": "Session inventory payload",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/SessionInventoryResponse" },
                                    "example": {
                                        "sessions": [
                                            {
                                                "sessionId": "claude-ses-1",
                                                "title": "扫描 Claude transcripts",
                                                "assistant": "claude-code",
                                                "progressState": "in_progress",
                                                "lastActivityAt": "2026-03-15T09:15:00Z",
                                                "projectPath": "C:/Projects/Claude Demo",
                                                "riskFlags": [],
                                                "controlAvailable": true,
                                                "valueScore": 86
                                            }
                                        ],
                                        "total": 1,
                                        "offset": 0,
                                        "limit": 20
                                    }
                                }
                            }
                        },
                        "401": {
                            "$ref": "#/components/responses/Unauthorized"
                        }
                    }
                }
            },
            "/api/v1/sessions/search": {
                "get": {
                    "summary": "Search session inventory",
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [
                        { "$ref": "#/components/parameters/Query" },
                        { "$ref": "#/components/parameters/AssistantFilter" },
                        { "$ref": "#/components/parameters/Limit" },
                        { "$ref": "#/components/parameters/Offset" },
                        {
                            "name": "sortBy",
                            "in": "query",
                            "schema": { "type": "string", "enum": ["score", "title", "assistant"] }
                        },
                        { "$ref": "#/components/parameters/Descending" }
                    ],
                    "responses": {
                        "200": {
                            "description": "Search hits",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/SessionSearchResponse" },
                                    "example": {
                                        "query": "Claude",
                                        "hits": [
                                            {
                                                "sessionId": "claude-ses-1",
                                                "title": "扫描 Claude transcripts",
                                                "assistant": "claude-code",
                                                "score": 120.0,
                                                "snippet": "扫描 Claude transcripts",
                                                "matchReasons": ["title"]
                                            }
                                        ],
                                        "total": 1,
                                        "offset": 0,
                                        "limit": 20
                                    }
                                }
                            }
                        },
                        "401": {
                            "$ref": "#/components/responses/Unauthorized"
                        }
                    }
                }
            },
            "/api/v1/sessions/{sessionId}": {
                "get": {
                    "summary": "Get session detail",
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [{ "$ref": "#/components/parameters/SessionId" }],
                    "responses": {
                        "200": {
                            "description": "Session detail",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/SessionDetail" }
                                }
                            }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "404": { "$ref": "#/components/responses/NotFound" }
                    }
                }
            },
            "/api/v1/sessions/{sessionId}/view": {
                "get": {
                    "summary": "Render session Markdown view",
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [{ "$ref": "#/components/parameters/SessionId" }],
                    "responses": {
                        "200": {
                            "description": "Markdown render bundle",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/SessionMarkdownView" },
                                    "example": {
                                        "sessionId": "claude-ses-1",
                                        "content": "# 扫描 Claude transcripts\n\n## Summary\n已定位项目目录并准备索引。"
                                    }
                                }
                            }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "404": { "$ref": "#/components/responses/NotFound" }
                    }
                }
            },
            "/api/v1/sessions/{sessionId}/expand": {
                "get": {
                    "summary": "Expand session context bundle",
                    "security": [{ "bearerAuth": [] }],
                    "parameters": [{ "$ref": "#/components/parameters/SessionId" }],
                    "responses": {
                        "200": {
                            "description": "Session context bundle",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/SessionExpandBundle" }
                                }
                            }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "404": { "$ref": "#/components/responses/NotFound" }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "Opaque token"
                }
            },
            "parameters": {
                "SessionId": {
                    "name": "sessionId",
                    "in": "path",
                    "required": true,
                    "schema": { "type": "string" }
                },
                "Query": {
                    "name": "query",
                    "in": "query",
                    "required": true,
                    "schema": { "type": "string" }
                },
                "AssistantFilter": {
                    "name": "assistant",
                    "in": "query",
                    "required": false,
                    "schema": { "type": "string" }
                },
                "Limit": {
                    "name": "limit",
                    "in": "query",
                    "required": false,
                    "schema": { "type": "integer", "minimum": 0 }
                },
                "Offset": {
                    "name": "offset",
                    "in": "query",
                    "required": false,
                    "schema": { "type": "integer", "minimum": 0 }
                },
                "SortBy": {
                    "name": "sortBy",
                    "in": "query",
                    "required": false,
                    "schema": {
                        "type": "string",
                        "enum": ["lastActivityAt", "title", "assistant", "valueScore"]
                    }
                },
                "Descending": {
                    "name": "descending",
                    "in": "query",
                    "required": false,
                    "schema": { "type": "boolean" }
                }
            },
            "responses": {
                "Unauthorized": {
                    "description": "Missing or invalid bearer token",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ErrorResponse" },
                            "example": { "error": "missing or invalid bearer token" }
                        }
                    }
                },
                "NotFound": {
                    "description": "Requested session was not found",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ErrorResponse" },
                            "example": { "error": "session not found: claude-ses-1" }
                        }
                    }
                }
            },
            "schemas": {
                "HealthResponse": {
                    "type": "object",
                    "required": ["status", "appName", "version"],
                    "properties": {
                        "status": { "type": "string" },
                        "appName": { "type": "string" },
                        "version": { "type": "string" }
                    }
                },
                "ErrorResponse": {
                    "type": "object",
                    "required": ["error"],
                    "properties": {
                        "error": { "type": "string" }
                    }
                },
                "SessionInventoryResponse": {
                    "type": "object",
                    "required": ["sessions", "total", "offset", "limit"],
                    "properties": {
                        "sessions": { "type": "array" },
                        "total": { "type": "integer" },
                        "offset": { "type": "integer" },
                        "limit": { "type": "integer" }
                    }
                },
                "SessionSearchResponse": {
                    "type": "object",
                    "required": ["query", "hits", "total", "offset", "limit"],
                    "properties": {
                        "query": { "type": "string" },
                        "hits": { "type": "array" },
                        "total": { "type": "integer" },
                        "offset": { "type": "integer" },
                        "limit": { "type": "integer" }
                    }
                },
                "SessionDetail": {
                    "type": "object",
                    "description": "Serialized dashboard session detail record"
                },
                "SessionMarkdownView": {
                    "type": "object",
                    "required": ["sessionId", "content"],
                    "properties": {
                        "sessionId": { "type": "string" },
                        "content": { "type": "string" }
                    }
                },
                "SessionExpandBundle": {
                    "type": "object",
                    "required": ["session", "relatedConfigs", "relatedAuditEvents", "transcriptHighlights", "todoItems", "keyArtifacts"],
                    "properties": {
                        "session": { "type": "object" },
                        "relatedConfigs": { "type": "array" },
                        "relatedAuditEvents": { "type": "array" },
                        "transcriptHighlights": { "type": "array" },
                        "todoItems": { "type": "array" },
                        "keyArtifacts": { "type": "array" }
                    }
                }
            }
        }
    })
}
