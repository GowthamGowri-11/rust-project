# RustFlow-AI API Documentation

## Base URL
```
http://localhost:8080/api/v1
```

## Endpoints

### Health Check
```
GET /api/v1/health
```
Returns system health status.

### Topology
```
GET /api/v1/topology
```
Returns network topology (nodes and links).

### Switches
```
GET /api/v1/switches
```
Returns list of connected OpenFlow switches.

### Flows
```
GET /api/v1/flows
```
Returns active flow rules.

### Metrics
```
GET /api/v1/metrics
```
Returns current network metrics.

### Optimize Routes
```
POST /api/v1/routes/optimize
```
Triggers route optimization.

### Prometheus Metrics
```
GET /metrics
```
Returns Prometheus-formatted metrics.

## Response Format

All responses are in JSON format:

```json
{
  "status": "success",
  "data": { ... }
}
```

## Error Responses

```json
{
  "status": "error",
  "message": "Error description"
}
```
